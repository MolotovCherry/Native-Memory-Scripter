# Function: find_demangled

Find a demangled iat entry in a module by name.

```admonish success title=""
This function is safe
```

### Parameters
- <code>module: [Module](../modules/objects-module.md)</code> - the module to search in.
- `name: str` - the iat function name to search for. case-sensitive fuzzy search.

### Exceptions
If module memory is invalid, failed to convert rva to va, or failed to get information from the module.

### Return Value
Returns a [`IATSymbol`](./objects-iatsymbol.md) if found, `None` if not found.
