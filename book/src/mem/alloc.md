# Function: alloc

Allocates `size` bytes of memory with protection flags [`Prot`](./objects-prot.md) in the calling process.

```admonish success title=""
This function is safe
```

### Parameters
- `size: int` - the size of memory to be allocated. If the size is 0, the function will allocate a full page of memory. If a specific size is provided, that amount of memory will be allocated, aligned to the next page size.
- <code>prot: [Prot](./objects-prot.md)</code> - the [protection flags](./objects-prot.md).

### Exceptions
If allocation failed.

### Return Value
Returns an [`Alloc`](./objects-alloc.md).
