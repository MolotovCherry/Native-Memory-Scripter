# Function: find_demangled

Returns the memory address of a demangled symbol. The search is fuzzy, but case-sensitive.


```admonish success title=""
This function is safe
```

### Parameters
- <code>module: [`Module`](../modules/objects-module.md)</code> - the module to search through.
- `name: str` - the name of the symbol to search for. case-sensitive fuzzy search. this means your search term matches by case-exactly, however it is a contains search, e.g. searching for `FooBar` in `void symbol FooBarBaz()` matches, but `foobar` won't match.

### Exceptions
If module memory is invalid, or cannot find any exports.

### Return Value
Returns an `int` representing the memory address of the symbol. If not found, returns `None`.
