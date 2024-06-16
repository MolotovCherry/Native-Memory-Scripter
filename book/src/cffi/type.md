# Module: Type

These types are meant to be used in [`NativeCall`](./objects-nativecall.md) and [`Callable`](./objects-callable.md).

## Floats
Python type: `int`

#### F32
#### F64

## Unsigned
Python type: `int`

#### U8
#### U16
#### U32
#### U64
#### U128

## Signed
Python type: `int`

#### I8
#### I16
#### I32
#### I64
#### I128

## Strings
Python type: `str`

#### CStr
null terminated
#### WStr
may or may not be null terminated, depends on api

## Chars
Python type: `str`

#### Char
1 byte
#### WChar
2 bytes

## Misc

#### Ptr
Python type: `int`

#### Bool
Python type: `boolean`

## Examples

```python
import cffi

def foo():
    pass

c = cffi.Callable(foo, cffi.Types.U64)
```
