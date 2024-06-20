# FAQ

## Why it keep crashing?
This is almost certainly the result of UB. See if you can check the following:

- are the function arguments correct?

- is the function return correct?

- is the function abi correct?

- are you following all safety invariants when reading, writing, and changing protections on memory?

- is the address you're hooking able to be hooked at the assembly level? not all addresses will work because of things like function size or where you placed the hook.

- if executing the trampoline fails, did you hook at a location where relocating old instructions to the trampoline works? for example, instructions which are relative to their location cannot be relocated.
