# cffi

This module allows you to call C functions from python, or use python callbacks for your hooks, as well as allowing you to call the trampoline.

## How?
This leverages a JIT compiler to create the needed machine code on the fly to wrap the C functions and call your python function, translating the arguments/return values in the process.

```admonish tip title="Python functions can JIT"
The bundled python interpreter is also compiled with JIT support to make your python functions run faster, though this may compile only under limited circumstances right now.

You must manually enable it. To make use of it, call `__jit__()` on any python function. If your function is uncompilable, you will quickly know as it'll throw an exception.
```
