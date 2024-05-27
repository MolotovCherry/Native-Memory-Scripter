# Function: data_scan

Searches for specific bytes in a memory region in the current process.

### Parameters
- `data: bytearray` - the bytes to search for.
- `addr: int` - the address to start the scan from.
- `scansize: int` - the maximum size of the scan, in bytes.

### Return Value
On success, it returns an `int` which represents the address of the first match found. On failure, it returns `None`.
