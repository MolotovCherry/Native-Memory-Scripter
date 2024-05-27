# Function: alloc_memory

Allocates `size` bytes of memory with protection flags [`prot`](./objects-prot.md) in the calling process.

```admonish success title=""
This function is safe
```

### Parameters
- `size: int` - The size of memory to be allocated. If the size is 0, the function will allocate a full page of memory. If a specific size is provided, that amount of memory will be allocated, aligned to the next page size.
- `prot: Prot` - The [protection flags](./objects-prot.md)

### Return Value
On success, it returns an `int` which represents the memory address of the allocation. On failure, it returns `None`.
