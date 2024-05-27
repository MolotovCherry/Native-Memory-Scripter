# Function: demangle_symbol

Demangles a symbol name, generally acquired from [enum_symbols](./enum-symbols.md).

NOTE: You might want to use [enum_symbols_demangled](./enum-symbols-demangled.md) or [find_symbol_address_demangled](./find-symbol-address-demangled.md).

### Parameters
- `symbol: str` - The mangled symbol name string

### Return Value
On success, it returns `str` which is the demangled symbol. On failure, it returns `None`.
