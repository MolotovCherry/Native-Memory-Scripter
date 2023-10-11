# BG3 Plugin Template

1. [Create a new plugin project from this template](https://github.com/new?template_name=BG3-Plugin-Template-Rust&template_owner=MolotovCherry)
2. Clone your repo with `git clone --recurse-submodules --remote-submodules <YourGitHubUrl>`

To build:
- [Install Rust](https://rustup.rs/)
- Install [Visual Studio](https://visualstudio.microsoft.com/downloads/) + Build tools + Desktop development in C++ + Windows SDK
- Build with `cargo build` or `cargo build --release`

[`libmem`](https://github.com/rdbo/libmem) Rust examples can be found [here](https://github.com/rdbo/libmem/tree/master/docs/examples/rust)

_Note: You are not required to use `libmem`! There are other libraries that exist which can do similar things_

## For mod program makers

For those who want to get a plugin's name/author/description/version, use the [BG3-Plugin-Lib](https://github.com/MolotovCherry/BG3-Plugin-Lib) (also compatible with C!)
