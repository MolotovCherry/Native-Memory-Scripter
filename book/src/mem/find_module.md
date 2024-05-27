# Function: find_module

Searches for a module in the calling process based on it's name or path.

```admonish success title=""
This function is safe
```

### Parameters
- `name: str` - string containing the name of the module, such as `"gamemodule.dll"`. It may also be a relative/absolute path, like `"bin/lib/gamemodule.dll"`.

### Return Value
On success, it returns [`module`](./objects-module.md); On failure, it returns `None`.
