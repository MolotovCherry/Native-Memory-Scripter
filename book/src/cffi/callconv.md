# Module: CallConv

These types are meant to be used in [`NativeCall`](./objects-nativecall.md) and [`Callable`](./objects-callable.md).

## Calling Conventions

### C
This is the default.

### WindowsFastcall

### Stdcall

### SystemV

## Example

~~~admonish example title=""
```python
import cffi

def foo():
    pass

c = cffi.Callable(foo, cffi.Types.U64, conv = cffi.CallConv.C)
```
~~~
