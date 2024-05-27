# Function: set_memory

Sets `size` bytes of `dst` as `byte` in the calling process.

```admonish danger title=""
This function is unsafe 🐉
```

### Parameters
- `dst: int` - virtual address that will be set to `byte` for `size` bytes.
- `byte: int` - the byte to set `size` bytes of `dst` as.
- `size: int` - the amount of bytes to set


### Return Value
On success, it returns `True`. On failure, it returns `False`.
