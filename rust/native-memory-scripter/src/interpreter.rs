use std::{
    fs,
    path::Path,
    sync::{Mutex, OnceLock},
    thread,
};

use eyre::{Context, Result};
use rustpython::InterpreterConfig;
use rustpython_vm::{
    builtins::PyStrRef, compiler, convert::ToPyObject, extend_class, prelude::*, py_class, Settings,
};
use serde::Deserialize;
use tracing::{error, info, info_span, trace};
use walkdir::WalkDir;

#[derive(Debug, Deserialize)]
struct Plugin {
    plugin: PluginDetails,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
struct PluginDetails {
    name: String,
    author: String,
    description: String,
    version: String,
}

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
        .min_depth(2)
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

        let (plugin, source, mut settings) = match entry.depth() {
            // subfolders - the file inside must be named main.py
            2 if path
                .file_name()
                .is_some_and(|f| f.eq_ignore_ascii_case("main.py")) =>
            {
                let parent = path.parent().unwrap().to_path_buf();
                let folder = parent.file_name().unwrap().to_string_lossy().to_string();

                let source = match fs::read_to_string(path) {
                    Ok(s) => s,
                    Err(error) => {
                        error!(%error, folder, "failed to read script");
                        continue;
                    }
                };

                let plugin_info = match fs::read_to_string(parent.join("plugin.toml")) {
                    Ok(v) => match toml::from_str::<Plugin>(&v) {
                        Ok(v) => v,
                        Err(error) => {
                            error!(%error, folder, "failed to deserialize plugin details");
                            continue;
                        }
                    },

                    Err(error) => {
                        error!(%error, folder, "unable to load plugin");
                        continue;
                    }
                };

                let mut settings = Settings::default();

                // add current directory to module path
                settings
                    .path_list
                    .push(parent.to_string_lossy().to_string());

                (plugin_info, source, settings)
            }

            _ => continue,
        };

        // add packages dir to module path
        settings
            .path_list
            .push(packages_dir.to_string_lossy().to_string());

        thread::spawn(move || {
            info!(
                "starting plugin: {} v{}",
                plugin.plugin.name, plugin.plugin.version
            );

            let span = info_span!("script", name = plugin.plugin.name);
            let _guard = span.enter();

            run_interpreter(settings, |vm| {
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

                Ok(())
            });
        });
    }

    Ok(())
}

fn run_interpreter<R>(settings: Settings, enter: impl FnOnce(&VirtualMachine) -> PyResult<R>) {
    let interp = InterpreterConfig::new()
        .settings(settings)
        .init_stdlib()
        .init_hook(Box::new(|vm| {
            use crate::modules::{
                cffi::cffi, hook::hook, info::info, mem::mem, modules::modules, scan::scan,
                segments::segments, symbols::symbols,
            };

            vm.add_native_module("mem".to_owned(), Box::new(mem::make_module));
            vm.add_native_module("info".to_owned(), Box::new(info::make_module));
            vm.add_native_module("cffi".to_owned(), Box::new(cffi::make_module));
            vm.add_native_module("hook".to_owned(), Box::new(hook::make_module));
            vm.add_native_module("modules".to_owned(), Box::new(modules::make_module));
            vm.add_native_module("scan".to_owned(), Box::new(scan::make_module));
            vm.add_native_module("segments".to_owned(), Box::new(segments::make_module));
            vm.add_native_module("symbols".to_owned(), Box::new(symbols::make_module));
        }))
        .interpreter();

    let res = interp.enter(|vm| {
        // add properties to the modules
        let mem = vm.import("mem", 0)?;
        let prot = crate::modules::mem::mem::_prot::make_module(vm);
        mem.set_attr("Prot", prot, vm)?;

        let cffi = vm.import("cffi", 0)?;
        let _type = crate::modules::cffi::cffi::_type::make_module(vm);
        cffi.set_attr("Type", _type, vm)?;
        let call_conv = crate::modules::cffi::cffi::_call_conv::make_module(vm);
        cffi.set_attr("CallConv", call_conv, vm)?;

        // Solve stdout/stderr stuff
        //
        vm.sys_module
            .set_attr("stdout", make_stdio(IoType::StdOut, vm), vm)?;

        vm.sys_module
            .set_attr("stderr", make_stdio(IoType::StdErr, vm), vm)?;

        // let scope = vm.new_scope_with_builtins();

        // let bootstrap = py_compile!(file = "src/modules/bootstrap.py");
        // let res = vm.run_code_obj(vm.ctx.new_code(bootstrap), scope);

        // if let Err(exc) = res {
        //     let mut data = String::new();
        //     vm.write_exception(&mut data, &exc)
        //         .map_err(|e| vm.new_runtime_error(e.to_string()))?;
        //     let data = data.trim();
        //     error!("Bootstrap error! This is a bug!\n{data}");
        // }

        if let Err(error) = enter(vm) {
            let mut data = String::new();
            if let Err(e) = vm.write_exception(&mut data, &error) {
                error!("failed to write error: {e}");
                return Ok(());
            }

            let data = data.trim();
            error!("\n{data}");
        }

        PyResult::Ok(())
    });

    if let Err(error) = res {
        interp.enter(|vm| {
            let mut data = String::new();
            if let Err(e) = vm.write_exception(&mut data, &error) {
                error!("Interpreter enter error: failed to write error: {e}");
                return;
            }

            let data = data.trim();
            error!("Interpreter enter error:\n{data}");
        });
    }
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
            let pos: Vec<char> = buffer.chars().collect();
            let pos = pos.iter().rposition(|x| *x == '\n');

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
