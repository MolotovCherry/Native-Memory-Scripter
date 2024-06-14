# Native Memory Scripter

Native Memory Scripter is a plugin which allows you to create a native plugin by writing Python scripts. This saves a lot of time and makes the process of native plugin dev a lot easier.

# Modders
If you'd like to write a native mod script, see the documentation [here](https://molotovcherry.github.io/Native-Memory-Scripter/)

# Features
- Use Python to manipulate memory
- Assembly of plaintext asm / disassembly of bytes
- Virtual method table hooking
- Module search, loading, and unloading
- Segment search (pages)
- External symbol search in modules with name demangling support
- Import address table search and hooking
- Pattern scanning for bytes in memory (supports AVX2, SSE4.2, and regular scalar scanning)
- Memory manipulation (read, write, set, and allocate memory)
- Call native functions from python, and hook (vmt, iat, and jmp) native functions with a python callback, along with trampoline to call the original
  - Powered by a JIT compiler
- Written in Rust 🦀

# Building

To build:
- [Install Rust](https://rustup.rs/)
- Install [Visual Studio](https://visualstudio.microsoft.com/downloads/) + Build tools + Desktop development in C++ + Windows SDK
- Build with `cargo build` or `cargo build --release`

# License
This software has a source-available non-open source license.
This software:
- may only be used for personal use
- may be forked, however modifications are not allowed
- may be compiled in its unmodified form
- modifications and compiling with modifications are allowed under the condition that you submit your changes back to the main repo
- cannot be redistributed
- cannot be used/re-used in your own projects
- cannot be used for any commercial purposes
- cannot be sold
- code cannot be used in any way for any purpose. it is copyrighted

For full terms, please see the [license](LICENSE)

# Disclaimer
THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
