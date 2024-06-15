# Function: data_scan

Scan for a given set of bytes starting at `address` for `scan_size` bytes.

```admonish danger title=""
This function is unsafe 🐉

- `address` must be a valid address for reads up to `scan_size`
- `scan_size` must be a valid length
```

### Parameters
- `data: bytearray` - the data to search for.
- `address: int` - the address to start looking from.
- `scan_size: int` - how many bytes to search for from the starting address.

### Return Value
On success, it returns an `int` representing the found location's memory address. On failure, it returns `None`.
