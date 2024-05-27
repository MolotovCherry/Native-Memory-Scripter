# Function: hook_code

Places a hook/detour onto the address `from`, redirecting it to the address `to` in the calling process. It returns a trampoline and the hook size; the trampoline can be used to call the original function.

```admonish danger title=""
This function is unsafe 🐉
```

### Parameters
- `from: int` - the address where the hook will be placed.
- `to: int` - the address where the hook will jump to.

### Return Value
On success, it returns `(trampoline_address: int, hook_size: int)`, where `trampoline_address` is an `int` where there is a trampoline/gateway that you can use to call the original function; `size` is the amount of bytes occupied by the hook (aligned to the nearest instruction). On failure, it returns `None`.
