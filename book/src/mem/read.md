# Function: read

Reads `size` bytes starting at `src` address into bytearray.

```admonish danger title=""
This function is unsafe 🐉

- `src` must be a valid address for reads up to `size`
```

### Parameters
- `src: int` - the base address to read from.
- `size: int` - the amount of bytes to read.

### Return Value
Returns a `bytearray` of the read bytes.
