# Function: assemble

Assembles a single instruction into machine code.

### Parameters
- `code: str` - a string of the instruction to be assembled. Example: `"jmp eax"`.

### Return Value
On success, it returns [`inst`](./objects-inst.md) containing the assembled instruction. On failure, it returns `None`.
