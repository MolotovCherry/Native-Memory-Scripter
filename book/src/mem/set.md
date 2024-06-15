# Function: set

Set `dst` address + `size` bytes to `byte`

```admonish danger title=""
This function is unsafe 🐉

- `dst` must be a valid address for writes up to `size`
```

### Parameters
- `dst: int` - the destination address to write to.
- `byte: int` - the byte to set the memory to.
- `size: int` - the amount of bytes from the `dst` to set.
