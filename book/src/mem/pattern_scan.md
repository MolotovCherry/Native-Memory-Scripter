# Function: pattern_scan

Searches for specific bytes with a mask filter in a memory region in the current process.

```admonish danger title=""
This function is unsafe 🐉
```

### Parameters
- `pattern: bytearray` - the bytes to search for (it is common practice to leave unknown bytes as 0).
- `mask: str` - a mask filter to apply to the pattern. Use 'x' for a known byte and '?' for an unknown byte. Example: `"xxxx???x?xxx"`.
- `addr: int` - the address to start the scan from.
- `scansize: int` - the maximum size of the scan, in bytes.

### Return Value
On success, it returns an `int` representing the address containing the first match found. On failure, it returns `None`.
