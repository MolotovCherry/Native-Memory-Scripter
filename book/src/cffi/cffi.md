# cffi

This module allows you to call C functions from python, or use python callbacks for your hooks, as well as allowing you to call the trampoline.

## How?
This leverages a JIT compiler to generate a machine code wrapper on the fly which has the same signature as the original native function. This wrapper then translates the native args to python, calls your python function with the args, and translates the return value back to native. In this way, it's transparent, just as if it were the real function. This also generates a trampoline to call the original code.

In the case of [`NativeCall`](./objects-nativecall.md) -- just the trampoline portion is used.

```admonish tip title="Python functions can JIT"
The bundled python interpreter is also compiled with JIT support to make your python functions run faster, though this may compile only under limited circumstances right now.

You must manually enable it. To make use of it, call `__jit__()` on any python function. If your function is uncompilable, you will quickly know as it'll throw an exception.
```
