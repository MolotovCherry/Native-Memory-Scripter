# Function: read_memory

Reads `size` bytes of memory in the calling process from a virtual address (`src`).

```admonish danger title=""
This function is unsafe 🐉
```

### Parameters
- `dst: int` - virtual address that will be set to `byte` for `size` bytes.
- `byte: int` - the amount of bytes to read


### Return Value
On success, it returns a `bytearray` containing the bytes read, and its length should be equal to size. On failure, it returns `None`.
