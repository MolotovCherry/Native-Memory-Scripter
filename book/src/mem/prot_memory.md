# Function: prot_memory

Changes the protection flag from `addr` for `size` bytes to the protection `prot` in the calling process. Returns the old protection flags.

```admonish danger title=""
This function is unsafe 🐉
```

### Parameters
- `addr: int` - the virtual address to change the protection flags.
- `size: int` - the size of the region to change the protection flags.
- <code>prot: [Prot](./objects-prot.md)</code> - the protection flags.


### Return Value
On success, it returns [`Prot`](./objects-prot.md) representing the old protection flags before changing. On failure, it returns `None`.
