# Function: demangle_symbol

Demangles a symbol name, generally acquired from [enum_symbols](./enum_symbols.md).

```admonish success title=""
This function is safe
```

```admonish info title="Note"
You might want to use [enum_symbols_demangled](./enum_symbols_demangled.md) or [find_symbol_address_demangled](./find_symbol_address_demangled.md).
```

### Parameters
- `symbol: str` - The mangled symbol name string.

### Return Value
On success, it returns `str` which is the demangled symbol. On failure, it returns `None`.
