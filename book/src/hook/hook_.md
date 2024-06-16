# Function: hook

Hooks an address by placing a jmp at the target and creating a trampoline to execute the original function.

```admonish danger title=""
This function is unsafe 🐉

- `from` must be a valid address which can be written to, and must be a valid location to write a jmp of 5 or 14 bytes at.
- `to` must be a valid target location, and must properly handle the requirements of the assembly at the jmp site.
```

```admonish note title=""
If the `from` address is within 32-bits of the `to` address, will write a 5 byte jmp, otherwise will write a 14 byte jmp.

```

### Parameters
- `from: int` - the address to hook.
- `to: int` - the address to redirect the `from` address to.

### Exceptions
If virtual protect fails, or fails to get the underlying code len.

### Return Value
Returns a [`Trampoline`](./objects-trampoline.md) which can be used to execute the original code at the hooked location.
