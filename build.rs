fn main() {
    // stamp dll with project metadata
    let mut res = winres::WindowsResource::new();

    // allow high dpi scaling
    //
    // the only reason this is here is because of the popup functionality,
    // and I use a 4k screen; I dislike pixelated text :(
    res.set_manifest(r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0" xmlns:asmv3="urn:schemas-microsoft-com:asm.v3">
    <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
        <application>
            <!-- Windows 10 and Windows 11 -->
            <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}"/>
        </application>
    </compatibility>
    <asmv3:application>
        <asmv3:windowsSettings>
        <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true</dpiAware>
        <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">system</dpiAwareness>
        </asmv3:windowsSettings>
    </asmv3:application>
</assembly>
"#);

    let _ = res.compile();

    // build and link libmem
    //
    // build times are long! it is recommended to cache these instead, and take the build artifacts generated
    // and hardcode this buildscript to your generated .lib / .dll file(s)
    // note that depending on which profile you build, debug or release, you will get debug or release
    // artifacts! make sure you cache and use the appropriate ones per profile!
    // you don't want to accidentally link to a debug .lib/.dll for your optimized release code

    let mut config = cmake::Config::new("libmem");

    config.generator("NMake Makefiles");
    config.build_target("libmem");

    let dst = config.build();

    println!("cargo:rustc-link-search=native={}\\build", dst.display());
    println!("cargo:rustc-link-lib=static=libmem");
}
