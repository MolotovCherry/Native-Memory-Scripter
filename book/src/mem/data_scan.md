# Function: data_scan

Searches for specific bytes in a memory region in the current process.

```admonish danger title=""
This function is unsafe 🐉
```

### Parameters
- `data: bytearray` - The bytes to search for.
- `addr: int` - The address to start the scan from.
- `scansize: int` - The maximum size of the scan, in bytes.

### Return Value
On success, it returns an `int` which represents the address of the first match found. On failure, it returns `None`.
