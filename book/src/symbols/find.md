# Function: find

Returns the memory address of a symbol. The search is case-sensitive and must be an exact match.


```admonish success title=""
This function is safe
```

### Parameters
- <code>module: [`Module`](../modules/objects-module.md)</code> - the module to search through.
- `name: str` - the name of the symbol to search for. case-sensitive exact search.

### Exceptions
If module memory is invalid, or cannot find any exports.

### Return Value
Returns an `int` representing the memory address of the symbol. If not found, returns `None`.
