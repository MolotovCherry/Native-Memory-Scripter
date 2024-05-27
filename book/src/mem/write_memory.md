# Function: write_memory

Writes `src` in the calling process into a virtual address (`dst`).

```admonish danger title=""
This function is unsafe 🐉
```

### Parameters
- `dst: int` - the address which will be written the bytes from `src`.
- `src: bytearray` - the bytes to write into `dst`.


### Return Value
On success, it returns `True`. On failure, it returns `False`.
