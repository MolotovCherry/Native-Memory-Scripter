# Function: find_symbol_address_demandled

Searches for a demangled symbol in a module, returning its virtual address.

```admonish success title=""
This function is safe
```

### Parameters
- <code>pmod: [module](./objects-module.md)</code> - The module where the symbol will be looked up from.
- `name: str` - The name of the demangled symbol to look up.

### Return Value
On success, it returns `int` representing the address of the demangled symbol; On failure, it returns `None`.
