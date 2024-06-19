# Object: NativeCall

Call a native function from Python.

## Constructor

- <code>address: int|[Symbol](../symbols/objects-symbol.md)</code> - The address or [`Symbol`](../symbols/objects-symbol.md) to call.
- `*args` - Any [`Type`](./type.md)'s matching the corresponding native function's argument types.
- `**kwargs` - `ret` for the [`Type`](./type.md) return value, and `conv` to change the [calling convention](./conv.md).

## Drop
```admonish danger title=""
The allocated code will automatically be freed when this is deleted or reclaimed.
```

## Magic
This object implements `__call__()`. You may call this object with your args and it will call the underlying native function.

```admonish danger title=""
Using the call function is unsafe 🐉

You must use the correct arguments / return types, otherwise calling this is ub.
```

## Example

~~~admonish example title=""
```python
import modules
import symbols
import cffi

module = modules.load("Dll1.dll")

create = symbols.find(module, "createTestClass")
one = symbols.find(module, "callTestMethod")
two = symbols.find(module, "callTestMethod2")

create = cffi.NativeCall(create, ret = cffi.Type.U64)
obj = create()

one = cffi.NativeCall(one, cffi.Type.U64)
one(obj)

two = cffi.NativeCall(two, cffi.Type.U64)
two(obj)
```
~~~
