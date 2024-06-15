# Function: disassemble

Dissasemble a single instruction at the target address

```admonish danger title=""
This function is unsafe 🐉

- The address must be valid for up to 16 bytes read
```

### Parameters
- `addr: int` - the starting address of the code to disassemble

### Exceptions
If it fails to diassemble

### Return Value
Returns an [`Inst`](objects-inst.md) representing the disassembled instruction
