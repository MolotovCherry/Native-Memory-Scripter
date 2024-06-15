# Function: load

Load a dll into the process.

```admonish success title=""
This function is safe
```

### Parameters
- `path: str` - the full path to the dll.

### Exceptions
If `LoadLibraryW` fails.

### Return Value
Returns a [`Module`](./objects-module.md).
