# Function: find_address_demangled

Returns the memory address of a demangled symbol. The search is fuzzy, but case-sensitive.


```admonish success title=""
This function is safe
```

### Parameters
- <code>module: [`Module`](./modules/module.md)</code> - the module to search through.
- `symbol_name: str` - the name of the symbol to search for. case-sensitive fuzzy search.

### Exceptions
If module memory is invalid, or cannot find any exports.

### Return Value
Returns an `int` representing the memory address of the symbol. If not found, returns `None`.
