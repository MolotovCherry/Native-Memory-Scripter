# How to get the library

1. Go to [libmem releases](https://github.com/rdbo/libmem/releases) and download the latest `libmem-*-x86_64-windows-msvc-static-mt.tar.gz`.

2. Grab `libmem.lib` from the `lib/release` directory and either place it in this `lib` directory, or place it in `C:\Program Files\libmem\lib` (the default libmem lib search path).
    - If you placed it in this lib folder, go to `.cargo/config.toml` and set `rustflags = ['-LC:\my\path\to\lib\folder']`

A note on `mt` vs `md`:
- Use `md` lib if rustflag `-Ctarget-feature=+crt-static` is not set
- Use `mt` lib if rustflag `-Ctarget-feature=+crt-static` is set

By default the static crt compile flag is set in `.cargo/config.toml`, so the instructions said to use `mt` version
