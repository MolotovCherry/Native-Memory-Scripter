# Function: sig_scan

Searches for a byte signature in a memory region in the current process.

```admonish danger title=""
This function is unsafe 🐉
```

### Parameters
- `sig: str` - string representation of a byte signature that can contain unknown bytes (`??`). Example: `"E9 ?? ?? ?? ?? 90 90 90 90"`.
- `addr: int` - the address to start the scan from.
- `scansize: int` - the maximum size of the scan, in bytes.


### Return Value
On success, it returns `int` representing the address of the first match found. On failure, it returns `None`.
