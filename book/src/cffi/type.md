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

#### CStr
Python type: `str`

When received as a [`Callback`](./objects-callable.md) argument or a [`NativeCall`](./objects-nativecall.md) return, it's a python string without null terminator. When used as a [`NativeCall`](./objects-nativecall.md) argument and [`Callback`](./objects-callable.md) return, you MUST include a null terminator in the string.

#### WStr
[`NativeCall`](./objects-nativecall.md) return type / [`Callback`](./objects-callable.md) argument: `int` (pointer to address)

[`NativeCall`](./objects-nativecall.md) argument / [`Callback`](./objects-callable.md) return type: [`WStr`](./objects-wstr.md)

This may or may not be null terminated, depends on api requirements. Some functions require you to pass a length, sometimes it needs to be null terminated. This is up to you to manage according to the api.

On a [`Callback`](./objects-callable.md), an argument of this type will be a ptr. On a [`NativeCall`](./objects-nativecall.md), a return of this type will be a ptr. In order to convert the ptr into a string you can use, see [`WStr`](./objects-wstr.md).

To return a `WStr` from a [`Callback`](./objects-callable.md), or give a `WStr` argument to a [`NativeCall`](./objects-nativecall.md), use [`WStr`](./objects-wstr.md).

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

This is a by-value struct (not a ptr to a struct). You can use this type in arg or return position to indicate receiving or returning a struct by value.

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
