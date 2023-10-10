fn main() {
    let mut config = cmake::Config::new("libmem");

    config.generator("NMake Makefiles");
    config.build_target("libmem");

    let dst = config.build();

    println!("cargo:rustc-link-search=native={}\\build", dst.display());
    println!("cargo:rustc-link-lib=static=libmem");
}
