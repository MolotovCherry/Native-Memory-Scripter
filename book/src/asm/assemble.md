# Function: assemble

Assembles asm instruction(s) from a string

```admonish success title=""
This function is safe
```

### Parameters
This function has multiple calling types.

#### First
Assemble a single instruction

- `code: str` - a string with the instruction to assemble, e.g. `jmp [rip]`

#### Second
Assemble multiple instructions (with runtime address)

- `code: str` - a string with the instructions to assemble, e.g. `jmp [rip]; nop`.
- `runtime_address: int` - the address to annotate each instruction with.

### Exceptions
If instruction fails to assemble

### Return Value
Returns an [`Inst`](objects-inst.md) representing the assembled instruction
