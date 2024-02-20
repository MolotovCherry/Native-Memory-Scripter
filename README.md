# BG3 Plugin Template

1. [Create a new plugin project from this template](https://github.com/new?template_name=BG3-Plugin-Template-Rust&template_owner=MolotovCherry)
2. Follow instructions in `lib` folder and read the README there to finish setting up `libmem`

To build:
- [Install Rust](https://rustup.rs/)
- Install [Visual Studio](https://visualstudio.microsoft.com/downloads/) + Build tools + Desktop development in C++ + Windows SDK
- Build with `cargo build` or `cargo build --release`

[`libmem`](https://github.com/rdbo/libmem) Rust examples can be found [here](https://github.com/rdbo/libmem/tree/master/docs/examples/rust)

Any build dll is compatible with the original [native mod loader](https://www.nexusmods.com/baldursgate3/mods/944) as well. Paths for config and logs are based off the dll's location to make it portable for users, regardless of loader used

_Note: You are not required to use `libmem`! There are other libraries that exist which can do similar things_

## For mod program makers

For those who want to get a plugin's name/author/description/version, use the [BG3-Plugin-Lib](https://github.com/MolotovCherry/BG3-Plugin-Lib) (also compatible with C!)
