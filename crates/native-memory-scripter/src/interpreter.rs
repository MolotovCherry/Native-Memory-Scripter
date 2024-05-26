use std::{fs, path::Path, thread};

use eyre::{Context, Result};
use rustpython::InterpreterConfig;
use rustpython_vm::{builtins::PyStrRef, compiler, extend_class, prelude::*, py_class, Settings};
use tracing::{error, info, info_span, trace};
use walkdir::WalkDir;

pub fn run_scripts(dll_dir: &Path) -> Result<()> {
    let scripts_dir = dll_dir.join("native-scripts");
    let packages_dir = scripts_dir.join("_packages");

    for dir in [&scripts_dir, &packages_dir, &packages_dir.join("libs")] {
        let name = dir.file_name().unwrap().to_string_lossy();

        if !dir.exists() {
            info!("creating {name} dir",);

            fs::create_dir(dir).context("failed to create dir")?;
        }

        if !dir.is_dir() {
            error!("{name} dir is not a directory. please manually fix this");
            return Ok(());
        }
    }

    let walk_dir = WalkDir::new(scripts_dir)
        .min_depth(1)
        .max_depth(2)
        .follow_links(true);

    for entry in walk_dir {
        let entry = match entry {
            Ok(d) => d,
            Err(error) => {
                error!(%error, "failed to read dir entry");
                continue;
            }
        };

        // skip all packages directory paths
        if entry.path().starts_with(&packages_dir) {
            continue;
        }

        let path = entry.path();

        trace!(path = %path.display(), "walking over entry");

        if path.is_file() {
            let (script, source, mut settings) = match entry.depth() {
                // immediate descendants - file can be named anything
                1 if path
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("py")) =>
                {
                    let script = path.file_stem().unwrap().to_string_lossy().to_string();

                    let source = match fs::read_to_string(path) {
                        Ok(s) => s,
                        Err(error) => {
                            error!(%error, script, "failed to read script");
                            continue;
                        }
                    };

                    let settings = Settings::default();

                    (script, source, settings)
                }

                // subfolders - the file inside must be named main.py
                2 if path
                    .file_name()
                    .is_some_and(|f| f.eq_ignore_ascii_case("main.py")) =>
                {
                    let parent = path.parent().unwrap().to_path_buf();
                    let script = parent.file_name().unwrap().to_string_lossy().to_string();

                    let source = match fs::read_to_string(path) {
                        Ok(s) => s,
                        Err(error) => {
                            error!(%error, name = script, "failed to read script");
                            continue;
                        }
                    };

                    let mut settings = Settings::default();

                    // add current directory to module path
                    settings
                        .path_list
                        .push(parent.to_string_lossy().to_string());

                    (script, source, settings)
                }

                _ => continue,
            };

            // add packages dir to module path
            settings
                .path_list
                .push(packages_dir.to_string_lossy().to_string());

            thread::spawn(move || {
                info!("starting script `{script}`");

                let span = info_span!("script", name = script);
                let _guard = span.enter();

                run_interpreter(settings, |vm| {
                    let result = (|| {
                        let scope = vm.new_scope_with_builtins();

                        let code_obj = vm
                            .compile(&source, compiler::Mode::Exec, "<main>".to_owned())
                            .map_err(|err| vm.new_syntax_error(&err, Some(&source)))?;

                        vm.run_code_obj(code_obj, scope)?;

                        PyResult::Ok(())
                    })();

                    if let Err(error) = result {
                        let mut data = String::new();
                        vm.write_exception(&mut data, &error).unwrap();
                        let data = data.trim();
                        error!("\n{data}");
                    }
                });
            });
        }
    }

    Ok(())
}

fn run_interpreter<R>(settings: Settings, enter: impl FnOnce(&VirtualMachine) -> R) -> R {
    InterpreterConfig::new()
        .settings(settings)
        .init_stdlib()
        .init_hook(Box::new(|_vm| {
            // vm.add_native_module(
            //     "your_module_name".to_owned(),
            //     Box::new(your_module::make_module),
            // );
        }))
        .interpreter()
        .enter(|vm| {
            vm.sys_module
                .set_attr("stdout", make_stdio(IoType::StdOut, vm), vm)
                .unwrap();

            vm.sys_module
                .set_attr("stderr", make_stdio(IoType::StdErr, vm), vm)
                .unwrap();

            enter(vm)
        })
}

enum IoType {
    StdOut,
    StdErr,
}

fn make_stdio(io: IoType, vm: &VirtualMachine) -> PyObjectRef {
    let ctx = &vm.ctx;

    let cls = PyRef::leak(py_class!(
        ctx,
        match io {
            IoType::StdOut => "PluginStdOut",
            IoType::StdErr => "PluginStdErr",
        },
        vm.ctx.types.object_type.to_owned(),
        {}
    ));

    let write_method = vm.new_method(
        "write",
        cls,
        move |_self: PyObjectRef, data: PyStrRef, _vm: &VirtualMachine| {
            let data = data.as_str();
            if !data.trim().is_empty() {
                match io {
                    IoType::StdOut => info!("{data}"),
                    IoType::StdErr => error!("{data}"),
                }
            }
        },
    );

    let flush_method = vm.new_method("flush", cls, |_self: PyObjectRef| {});
    extend_class!(ctx, cls, {
        "write" => write_method,
        "flush" => flush_method,
    });

    ctx.new_base_object(cls.to_owned(), None)
}
