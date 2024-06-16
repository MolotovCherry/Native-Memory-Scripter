# hook

This module allows you to use regular jmp hooks.

## How?

The `from` address is replaced with a 5 or 14 byte jmp (depending on whether target address is within 32-bits or not).

The old code that was replaced is placed at the beginning of the trampoline, and a jmp is made back to the original function, but right after the original jmp we placed.

The custom hook function is free to call the trampoline if it wishes to.

The jmp can actually be placed anywhere, not necessarily at the beginning of the function. But you will have to make sure that this works properly in the assembly.

<img data-rsrc="/_assets/hook-$theme.svg"/>

```admonish warning title="Not all instructions can be replaced"
Certain instructions cannot be relocated to the trampoline. For example, instructions which use a relative address require themselves to be at the original address. It is your job to ensure the location that gets replaced is capable of being replaced. Or you could also just not call the trampoline.
```
