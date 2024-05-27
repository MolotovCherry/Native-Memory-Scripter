# Function: enum_symbols_demangled

Enumerates all the demangled symbols in a module.

```admonish success title=""
This function is safe
```

### Parameters
- <code>pmod: [module](./objects-module.md)</code> - module which the demangled symbols will be searched from.

### Return Value
On success, it returns <code>[[symbol](./objects-symbol.md)]</code>; On failure, it returns `None`.
