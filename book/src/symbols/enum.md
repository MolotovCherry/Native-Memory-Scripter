# Function: enum

Returns a list of all symbols for a [`Module`](../modules/objects-module.md).


```admonish success title=""
This function is safe
```

### Parameters
- <code>module: [`Module`](../modules/objects-module.md)</code> - the module to get all symbols from.

### Exceptions
If module memory is invalid, or cannot find any exports.

### Return Value
Returns a <code>[[`Symbol`](./objects-symbol.md)]</code>.
