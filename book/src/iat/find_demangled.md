# Function: find_demangled

Finds an [`IATSymbol`](./objects-iatsymbol.md) in a module [`Module`](../modules/objects-module.md) and demangles its name.

```admonish success title=""
This function is safe
```

### Parameters
This function has two calling signatures.

#### Name
- <code>module: [`Module`](../modules/objects-module.md)</code> - the module to get the symbols for.
- `name: str` - the symbol name to look for. must be exact case-sensitive match.

#### Name and Dll name
- <code>module: [`Module`](../modules/objects-module.md)</code> - the module to get the symbols for.
- `dll_name: str` - the dll name to look for the symbol in. is an exact case sensitive match, e.g. `fooBar.dll`.
- `name: str` - the symbol name to look for. is a case-sensitive exact match.

### Exceptions
If module in memory is invalid or cannot otherwise be read.

### Return Value
Returns a [`IATSymbol`](./objects-iatsymbol.md) if found, or `None` if not found.
