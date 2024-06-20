# Function: disassemble

Dissasemble instructions.

### Parameters
This function has multiple calling types.

#### First (address)
Disassemble single instruction at address.

```admonish danger title=""
This call is unsafe 🐉

Address must be valid for up to 16 bytes read.
```

- `address: int` - the starting address of the code to disassemble.

#### Second (address)
Disassemble all instructions from an address, with a runtime address.

```admonish danger title=""
This call is unsafe 🐉

Address must be valid for up to size read.
```

- `address: int` - the starting address of the code to disassemble.
- `size: int` - how many bytes into the address to read.
- `runtime_address: int` - the address to annotate each instruction with.

#### Third (address)
Disassemble `count` instructions from address.

```admonish danger title=""
This call is unsafe 🐉

Address must be valid for up to size read.
```

- `address: int` - the starting address of the code to disassemble.
- `size: int` - how many bytes into the address to read.
- `runtime_address: int` - the address to annotate each instruction with.
- `count: int` - how many instructions to disassemble.

#### First (bytes)
Disassemble all bytes from into instructions.

```admonish success title=""
This call is safe
```

- `bytes: bytes` - the bytes to disassemble.

#### Second (bytes)
Disassemble all bytes from into instructions, with runtime address.

```admonish success title=""
This call is safe
```

- `bytes: bytes` - the bytes to disassemble.
- `runtime_address: int` - the address to annotate each instruction with.

#### Third (bytes)
Disassemble `count` instructions from address.

```admonish success title=""
This call is safe
```

- `bytes: bytes` - the bytes to disassemble.
- `runtime_address: int` - the address to annotate each instruction with.
- `count: int` - how many instructions to disassemble.

### Exceptions
If it fails to diassemble

### Return Value
Returns an [`Inst`](objects-inst.md) representing the disassembled instruction

## Example

~~~admonish example title=""
```python
import asm

foo = bytes.fromhex('00 00 00 00')
insts = asm.disassemble(foo)

# pretty print each instruction
for i in insts:
    print(i)

```
~~~
