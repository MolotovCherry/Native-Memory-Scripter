# Function: prot

Change the protection of a block of memory.

```admonish danger title=""
This function is unsafe 🐉
```

### Parameters
- `address: int` - the base address to change the protection on.
- `size: int` - how many bytes from the base address to change.
- <code>prot: [`Prot`](./objects-prot.md)</code> - the protection flag.

### Exceptions
If changing the protection failed.

### Return Value
The old [`Prot`](./objects-prot.md).
