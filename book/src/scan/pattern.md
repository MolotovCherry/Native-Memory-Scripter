# Function: pattern

Searches `address` for `scan_size` bytes for a match given some bytes and a pattern.

```admonish danger title=""
This function is unsafe 🐉

- `address` must be a valid address for reads up to `scan_size`
- `scan_size` must be a valid length
```

### Parameters
- `pattern: bytearray` - the data to search for. if some bytes are masked out, it's customary to leave them at `0`.
- `mask: str` - the mask for the bytes. use `x` for a known byte and `?` for an unknown byte. example, `xx?x?xx?`
- `address: int` - the starting address to look for the pattern at.
- `scan_size: int` - how many bytes to search for from the starting address.

### Return Value
On success, it returns an `int` representing the found location's memory address. On failure, it returns `None`.
