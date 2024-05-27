# Function: alloc_memory

Allocates `size` bytes of memory with protection flags [`prot`](./objects-prot.md) in the calling process.

### Parameters
- `size: int` - the size of the region to change the protection flags
- `prot: Prot` - the [protection flags](./objects-prot.md)

### Return Value
On success, it returns an `int` which represents the memory address of the allocation. On failure, it returns `None`.
