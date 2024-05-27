# Function: free_memory

Frees `size` bytes of allocated memory in the calling process.

```admonish danger title=""
This function is unsafe 🐉
```

### Parameters
- `alloc: int` - virtual address of the allocated memory.
- `size: int` - the size of the region to deallocate.

### Return Value
On success, it returns `True`; On failure, it returns `False`.
