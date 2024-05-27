# Function: load_module

Loads a module into the calling process from its path.

```admonish success title=""
This function is safe
```

### Parameters
- `modpath: str` - string containing a relative/absolute path, like `"bin/lib/gamemodule.dll"`.

### Return Value
On success, it returns [`module`](./objects-module.md) which contains information about the loaded module. On failure, it returns `None`.
