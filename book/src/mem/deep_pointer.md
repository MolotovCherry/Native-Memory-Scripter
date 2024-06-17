# Function: deep_pointer

Calculates a deep pointer address by applying a series of offsets to a base address and dereferencing intermediate pointers.

```admonish danger title=""
This function is unsafe 🐉
```

### Parameters
- `base: int` - the base address to start at.
- `offsets: [int]` - the offsets used to navigate through the memory addresses. must be unsigned.

### Exceptions
If memory address is null, or no offsets were provided.

### Return Value
An `int` representing the final address.
