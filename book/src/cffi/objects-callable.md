# Object: Callable

Hook a function and use a python function as the hooks callback.

## Constructor

- `Callable[[*args], Any]` - Any args and Any return type. Constrained to the types available in [`Type`](./type.md). Must match the following signature passed in to the constructor.
- `*args` - Any [`Type`](./type.md)'s matching the corresponding native function's argument types.
- `**kwargs` - `ret` for the [`Type`](./type.md) return value, and `conv` to change the [calling convention](./conv.md).

## Drop
```admonish danger title=""
The allocated callback code and trampoline code will automatically be freed when this is deleted or reclaimed.
```

## Properties

#### address: int
The address of the jitted callback function (the code that is a stand-in replacement of a native function and calls your python callback).

#### code_size: int
The size of the jitted callback function in bytes.

#### trampoline_address:
The address of the [underlying trampoline](../hook/objects-trampoline.md) (the trampoline fn that the jitpoline calls).

#### trampoline_size
The size of the [underlying trampoline](../hook/objects-trampoline.md).

#### jitpoline_address
The address of the jitpoline (the jitted trampoline that `__call__` calls).

## Magic
This object implements `__call__()`. You may call this object with your args and it will call the trampoline.

```admonish danger title=""
Using the call function is unsafe 🐉

You must use the correct arguments / return types, otherwise using the function will be ub.

Additionally, your callback function MUST gracefully handle all possible exceptions and return something. If it there's an uncaught exception, it is UB. But to protect the program, it will instantly crash instead. You should fix it asap.

If you specified a return type, you MUST always return a value of that type, even if your function caught an exception.
```

## Methods

### hook
`jmp` hook `from` address. This will attempt to allocate within ± 2GB of `from` address so it can use a `5` byte `jmp`, but if it's unable to it will use `14` byte `jmp`.

```admonish danger title=""
This function is unsafe 🐉

- `from` must point to a `xr` function with the same signature as your callable (abi, parameters, and return).
```

- <code>from: int|[Symbol](../symbols/objects-symbol.md)</code> - the function address or [`Symbol`](../symbols/objects-symbol.md) to hook.

### hook_iat
Hook an import address table entry.

```admonish danger title=""
This function is unsafe 🐉
```

- <code>entry: [IATSymbol](../iat/objects-iatsymbol.md)</code> - the iat symbol to hook.

#### Exceptions
If virtual protect fails.

### hook_vmt
Hook a virtual method table entry.

```admonish danger title=""
This function is unsafe 🐉

- `index` must be a valid index, and abi, args, and return must all be correct types.
```

- <code>vtable: [VTable](../vmt/objects-vtable.md)</code> - the vtable within which to hook.
- `index: int` - the index of the vtable method to hook.

#### Exceptions
If virtual protect fails.

### unhook
Unhook the callback.

```admonish danger title=""
This function is unsafe 🐉
```

#### Exceptions
If virtual protect fails.

## Example

~~~admonish example title=""
```python
import modules
import symbols
import cffi

# this will be called every time the original code calls "createTestClass"
def foo():
    # call the trampoline
    val = callable(obj)
    return val

module = modules.load("Dll1.dll")

create = symbols.find(module, "createTestClass")

callable = cffi.Callable(foo, ret = cffi.Type.U64)
callable.hook(create)

# unhook the callback
callable.unhook()
```
~~~
