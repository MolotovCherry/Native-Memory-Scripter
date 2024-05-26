use std::{
    fs,
    path::Path,
    sync::{Mutex, OnceLock},
    thread,
};

use eyre::{Context, Result};
use rustpython::InterpreterConfig;
use rustpython_vm::{
    builtins::PyStrRef, compiler, convert::ToPyObject, extend_class, prelude::*, py_class,
    py_compile, Settings,
};
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

        if !path.is_file() {
            continue;
        }

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

                    scope.globals.set_item(
                        "__name__",
                        vm.ctx.new_str("__main__").as_object().to_pyobject(vm),
                        vm,
                    )?;

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

    Ok(())
}

fn run_interpreter<R>(settings: Settings, enter: impl FnOnce(&VirtualMachine) -> R) -> R {
    InterpreterConfig::new()
        .settings(settings)
        .init_stdlib()
        .init_hook(Box::new(|vm| {
            use crate::modules::mem::mem as mem_mod;

            vm.add_native_module("mem".to_owned(), Box::new(mem_mod::make_module));
        }))
        .interpreter()
        .enter(|vm| {
            vm.sys_module
                .set_attr("stdout", make_stdio(IoType::StdOut, vm), vm)
                .unwrap();

            vm.sys_module
                .set_attr("stderr", make_stdio(IoType::StdErr, vm), vm)
                .unwrap();

            let scope = vm.new_scope_with_builtins();

            let bootstrap = py_compile!(file = "src/modules/bootstrap.py");
            let res = vm.run_code_obj(vm.ctx.new_code(bootstrap), scope);

            if let Err(exc) = res {
                let mut data = String::new();
                vm.write_exception(&mut data, &exc).unwrap();
                let data = data.trim();
                error!("Bootstrap error! This is a bug!\n{data}");
            }

            enter(vm)
        })
}

#[derive(Copy, Clone)]
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

    static STDOUT_BUFFER: OnceLock<Mutex<String>> = OnceLock::new();
    static STDERR_BUFFER: OnceLock<Mutex<String>> = OnceLock::new();

    STDOUT_BUFFER.get_or_init(|| Mutex::new(String::new()));
    STDERR_BUFFER.get_or_init(|| Mutex::new(String::new()));

    let write_method = vm.new_method(
        "write",
        cls,
        move |_self: PyObjectRef, data: PyStrRef, _vm: &VirtualMachine| {
            let buffer = match io {
                IoType::StdOut => STDOUT_BUFFER.get().unwrap(),
                IoType::StdErr => STDERR_BUFFER.get().unwrap(),
            };

            let mut buffer = buffer.lock().unwrap();

            let data = data.as_str();

            buffer.push_str(data);
            let pos = buffer.chars().position(|x| x == '\n');
            if let Some(pos) = pos {
                let slice = buffer.drain(..=pos).collect::<String>();
                let slice = slice.trim_end_matches(|x| x == '\r' || x == '\n');
                match io {
                    IoType::StdOut => info!("{slice}"),
                    IoType::StdErr => error!("{slice}"),
                }
            }
        },
    );

    let flush_method = vm.new_method("flush", cls, move |_self: PyObjectRef| {
        let buffer = match io {
            IoType::StdOut => STDOUT_BUFFER.get().unwrap(),
            IoType::StdErr => STDERR_BUFFER.get().unwrap(),
        };

        let mut buffer = buffer.lock().unwrap();

        let data = buffer.drain(..).collect::<String>();
        let slice = data.trim_end_matches(|x| x == '\r' || x == '\n');

        if !slice.is_empty() {
            match io {
                IoType::StdOut => info!("{slice}"),
                IoType::StdErr => error!("{slice}"),
            }
        }
    });

    extend_class!(ctx, cls, {
        "write" => write_method,
        "flush" => flush_method,
    });

    ctx.new_base_object(cls.to_owned(), None)
}
