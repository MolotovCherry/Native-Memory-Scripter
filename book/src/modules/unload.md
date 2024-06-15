# Function: unload

Unload a module from the process.

```admonish success title=""
This function is safe
```

```admonish info title=""
Modules are only ever unloaded from a process once their internal refcount reaches 0. This only decreases the refcount by 1.
```

```admonish warning title=""
Don't repeatedly call this without reason, or you will unload the module from memory prematurely when it's still being used 🐉.
```

### Parameters
- `path: str` - the full path to the dll.

### Exceptions
If `GetModuleHandleW` or `FreeLibrary` fails.
