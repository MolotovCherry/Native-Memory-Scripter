# Function: demangle

Demangle a symbol.

Supports C++ (GCC-style compilers and MSVC), Rust (both legacy and v0), Swift (up to Swift 5.3), and ObjC (only symbol detection).

```admonish success title=""
This function is safe
```

### Parameters
- `name: str` - the mangled symbol to demangle.

### Return Value
On success, it returns `str` representing the demangled symbol. On failure, it returns `None`.
