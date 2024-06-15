# Function: enum_demangled

Return a demangled list of import address table enties for a given [`Module`](../modules/objects-module.md)

```admonish success title=""
This function is safe
```

### Exceptions
If module memory is invalid, failed to convert rva to va, or failed to get information from the module.

### Return Value
Returns a <code>[[`IATSymbol`](./objects-iatsymbol.md)]</code>.
