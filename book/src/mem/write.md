# Function: write

Write some bytes to `dst`.

```admonish danger title=""
This function is unsafe 🐉

- `dst` must be a valid address for writes up to size of bytearray.
```

### Parameters
- `src: bytearray` - the bytes to write to target.
- `dst: int` - the destination address to write to.
