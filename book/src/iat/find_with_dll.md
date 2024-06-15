# Function: find_with_dll

Return a list of all modules

```admonish success title=""
This function is safe
```

### Exceptions
If unable to create a snapshot, no modules exist, unable to convert from utf16 to utf8, or load library.

### Return Value
Returns a <code>[[`Module`](./objects-module.md)]</code> containing all of the process's modules.
