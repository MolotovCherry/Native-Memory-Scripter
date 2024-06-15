# Function: enum_demangled

Returns a list of all demangled symbols for a [`Module`](./modules/module.md).


```admonish success title=""
This function is safe
```

### Parameters
- <code>module: [`Module`](./modules/module.md)</code> - the module to get all demangled symbols from.

### Exceptions
If module memory is invalid, or cannot find any exports.

### Return Value
Returns a <code>[[`Symbol`](./objects-symbol.md)]</code>.
