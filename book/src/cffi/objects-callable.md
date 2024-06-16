# Object: Callable

Hook a function and use a python function as the hooks callback.

## Constructor

- `Callable[[*args], Any]` - Any args and Any return type. Constrained to the types available in [`Type`](./type.md). Must match the following signature passed in to the constructor.
- `*args` - Any [`Type`](./type.md)'s matching the corresponding native function's argument types.
- `**kwargs` - `ret` for the [`Type`](./type.md) return value, and `conv` to change the [calling convention](./callconv.md).

## Drop
```admonish danger title=""
The allocated callback code and trampoline code will automatically be freed when this is deleted or reclaimed.
```

## Properties

#### address: int
The address of the callback function.

#### code_size: int
The size of the callback function in bytes.

## Magic
This object implements `__call__()`. You may call this object with your args and it will call the trampoline.

## Methods

### hook
`jmp` hook `from` address.

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
