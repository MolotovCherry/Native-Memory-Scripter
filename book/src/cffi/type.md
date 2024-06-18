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
Python type: `str`

This must be null terminated

#### WStr
Argument type: `int` (pointer to address)
Return type: `bytes`

This may or may not be null terminated, depends on api requirements.

 ```py
 s = "Hello world!"
 utf16_str = s.encode('utf-16')
 ```

## Chars
Python type: `str`

#### Char
Python type: `str` (1 character that can fit in a `u8`)

1 byte

#### WChar
Python type: `str` (1 character that can fit in a `u16`)

2 bytes

## Misc

#### Ptr
Python type: `int`

#### Bool
Python type: `boolean`

#### Struct(size: int)
Python type: `bytes`

You can use this type in arg or return position to indicate receiving or returning a struct by value.

```admonish danger title=""
The size must be correct or it'll be ub.
```

## Example

~~~admonish example title=""
```python
import cffi

def foo(arg: int):
    pass

c = cffi.Callable(foo, cffi.Types.U64)
```
~~~
