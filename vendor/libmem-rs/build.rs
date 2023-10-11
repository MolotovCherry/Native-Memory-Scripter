use std::error::Error;
use std::path::PathBuf;

use winreg::enums::HKEY_LOCAL_MACHINE;
use winreg::RegKey;

pub fn get_windows_kits_dir() -> Result<PathBuf, Box<dyn Error>> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = r"SOFTWARE\Microsoft\Windows Kits\Installed Roots";
    let dir: String = hklm.open_subkey(key)?.get_value("KitsRoot10")?;

    Ok(dir.into())
}

/// Retrieves the path to the user mode libraries. The path may look something like:
/// `C:\Program Files (x86)\Windows Kits\10\lib\10.0.18362.0\um`.
pub fn get_um_dir() -> Result<PathBuf, Box<dyn Error>> {
    // We first append lib to the path and read the directory..
    let dir = get_windows_kits_dir()?.join("Lib").read_dir()?;

    // In the lib directory we may have one or more directories named after the version of Windows,
    // we will be looking for the highest version number.
    let mut dir = dir
        .filter_map(Result::ok)
        .map(|dir| dir.path())
        .filter(|dir| {
            dir.components()
                .last()
                .and_then(|c| c.as_os_str().to_str())
                .map_or(false, |c| c.starts_with("10.") && dir.join("um").is_dir())
        })
        .max()
        .ok_or_else(|| "not found")?;

    dir.push("um");
    dir.push("x64");

    // Finally append um to the path to get the path to the user mode libraries.
    Ok(dir)
}

fn main() {
    // build and link libmem
    //
    // build times are long! it is recommended to cache these instead, and take the build artifacts generated
    // and hardcode this buildscript to your generated .lib file

    let mut config = cmake::Config::new("libmem");

    config.generator("NMake Makefiles");
    config.define("LIBMEM_BUILD_TESTS", "OFF");
    config.define("LIBMEM_BUILD_STATIC", "ON");
    // Build erorrs out in debug mode, recommended to cache artifacts
    config.define("CMAKE_BUILD_TYPE", "Release");
    config.build_target("libmem");

    let dst = config.build();

    let build_path = dst.join("build");

    // libmem.lib, llvm.lib
    println!("cargo:rustc-link-search=native={}", build_path.display());
    // keystone.lib
    println!(
        r"cargo:rustc-link-search=native={}\keystone-engine-prefix\src\keystone-engine-build\llvm\lib",
        build_path.display()
    );
    // capstone.lib
    println!(
        r"cargo:rustc-link-search=native={}\capstone-engine-prefix\src\capstone-engine-build",
        build_path.display()
    );
    // LIEF.lib
    println!(
        r"cargo:rustc-link-search=native={}\lief-project-prefix\src\lief-project-build",
        build_path.display()
    );
    println!("cargo:rustc-link-lib=static=keystone");
    println!("cargo:rustc-link-lib=static=capstone");
    println!("cargo:rustc-link-lib=static=LIEF");
    println!("cargo:rustc-link-lib=static=llvm");
    println!("cargo:rustc-link-lib=static=libmem");

    // user32.lib, psapi.lib, ntdll.lib
    println!(
        "cargo:rustc-link-search=native={}",
        get_um_dir().unwrap().display()
    );

    println!("cargo:rustc-link-lib=static=user32");
    println!("cargo:rustc-link-lib=static=psapi");
    println!("cargo:rustc-link-lib=static=ntdll");
}
