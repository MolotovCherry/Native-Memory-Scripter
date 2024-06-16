# Function: enum

Returns a list of all import address table entries in a [`Module`](../modules/objects-module.md).

```admonish success title=""
This function is safe
```

### Parameters
- <code>module: [`Module`](../modules/objects-module.md)</code> - the module to get the symbols for.

### Exceptions
If module in memory is invalid or cannot otherwise be read.

### Return Value
Returns a <code>[[IATSymbol](./objects-iatsymbol.md)]</code>
