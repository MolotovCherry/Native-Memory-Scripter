# Function: assemble_ex

Assembles all asm instructions from string with a runtime address

```admonish success title=""
This function is safe
```

### Parameters
- `code: str` - A string with the instruction to assemble, e.g. `jmp [rip]`
- `address: int` - An optional address offset to generate each [`Inst`](objects-inst.md)'s starting address in the output

### Exceptions
If instructions fail to assemble

### Return Value
Returns a <code>[[`Inst`](objects-inst.md)]</code> representing the assembled instructions
