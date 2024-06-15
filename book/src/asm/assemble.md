# Function: assemble

Assembles a single asm instruction from a string

```admonish success title=""
This function is safe
```

### Parameters
- `code: str` - A string with the instruction to assemble, e.g. `jmp [rip]`

### Exceptions
If instruction fails to assemble

### Return Value
Returns an [`Inst`](objects-inst.md) representing the assembled instruction
