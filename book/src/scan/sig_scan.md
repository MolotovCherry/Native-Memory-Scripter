# Function: sig_scan

Searches `address` for `scan_size` bytes for a given IDA-style signature.

```admonish danger title=""
This function is unsafe 🐉

- `address` must be a valid address for reads up to `scan_size`
- `scan_size` must be a valid length
```

### Parameters
- `sig: str` - an IDA-style signature to search for, e.g. `11 22 33 ?? 44 ?? 55 ?? ??`, where `11` is a known byte and `??` is an unknown byte.
- `address: int` - the starting address to look for the pattern at.
- `scan_size: int` - how many bytes to search for from the starting address.

### Return Value
On success, it returns an `int` representing the found location's memory address. On failure, it returns `None`.
