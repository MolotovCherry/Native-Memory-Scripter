# Function: find_process

Searches for a process based on its name or path.

```admonish success title=""
This function is safe
```

### Parameters
- `procstr: str` - string containing the name of the process, such as `"test1.exe"`. It may also be a relative/absolute path, like `"mygame/game.exe"`.

### Return Value
On success, it returns [`process`](./objects-process.md); On failure, it returns `None`.
