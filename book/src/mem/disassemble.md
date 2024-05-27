# Function: disassemble

Disassembles a single instruction into an [`inst`](./objects-inst.md)

### Parameters
- `code: int` - virtual address of the instruction to be disassembled.

### Return Value
On success, it returns an [`inst`](./objects-inst.md) which is the disassembled instruction. On failure, it returns `None`.
