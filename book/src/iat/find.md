# Function: find

Finds an [`IATSymbol`](./objects-iatsymbol.md) in a module [`Module`](../modules/objects-module.md).

```admonish success title=""
This function is safe
```

### Parameters
This function has two calling signatures.

#### Name / Ordinal
- <code>module: [`Module`](../modules/objects-module.md)</code> - the module to get the symbols for.
- `name: str|u16` - the symbol name or ordinal number to look for. must be exact case-insensitive match.

#### Name / Ordinal and Dll name
- <code>module: [`Module`](../modules/objects-module.md)</code> - the module to get the symbols for.
- `dll_name: str` - the dll name to look for the symbol in. is an exact case insensitive match, e.g. `fooBar.dll`.
- `name: str|u16` - the symbol name or ordinal number to look for. must be exact case-insensitive match.

### Exceptions
If module in memory is invalid or cannot otherwise be read.

### Return Value
Returns a [`IATSymbol`](./objects-iatsymbol.md) if found, or `None` if not found.
