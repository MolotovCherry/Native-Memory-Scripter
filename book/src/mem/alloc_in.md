# Function: alloc_in

Allocates `size` bytes of memory with protection flags [`Prot`](./objects-prot.md) in the calling process.

```admonish success title=""
This function is safe
```

### Parameters
- `begin_addr: int` - the beginning address of the region you want to allocate inside. this will be rounded up to the next system allocation granularity.
- `end_addr: int` - the ending address of the region you want to allocate inside. this will be rounded down to previous system allocation granularity.
- `size: int` - the size of memory to be allocated.
- `align: int` - the power-of-2 alignment. align must be 0, or power-of-2, >= system allocation granularity (seems to be `65536` afaict), and must be a multiple of system allocation granularity. specifying 0 automatically aligns on the system allocation granularity.
- <code>prot: [Prot](./objects-prot.md)</code> - the [protection flags](./objects-prot.md).

### Exceptions
If allocation failed. If `begin_addr >= end_addr `. If `align` is not power of 2, less than system allocation granularity (page boundary), or not a multiple of system allocation granularity. If begin addresses next rounded up to granularity is not within begin..end. If end address rounded down to granularity is not within begin..end. If begin and/or end addresses are outside of minimum/maximum application address.

### Return Value
Returns an [`Alloc`](./objects-alloc.md).
