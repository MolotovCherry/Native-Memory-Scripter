# Object: WStr

Converts a [`WStr`](./type.md#wstr) python string to a wstr pointer, or a pointer to a python string.

## Drop
```admonish danger title=""
Memory is freed when this object is deleted or reclaimed by gc.
```

## Constructor

This constructor has two ways to call it. One way to convert a python string to `WStr`, and another way to convert an address to a python string.

### Python string to `WStr`
When used this way, it converts your python string to a type that you can return in a [`Callable`](./objects-callable.md) or give as an argument to a [`NativeCall`](./objects-nativecall.md).

```admonish success title=""
This call is safe
```

- `data: str` - the string to convert to `WStr`. do not put a null terminator in it, use the kwarg for that.
- `**kwargs` - `null: boolean` - insert a null terminator at the end of your string.

### Address to `WStr`
When used this way, it converts a `WStr` pointer to this type, allowing you to access it as a python string.

Use either one of the `null` or `len` kwargs to tell this type how to decode the string from the address. Only 1 of these is allowed, do not use both kwargs.

```admonish danger title=""
This call is unsafe 🐉

- `address` must be a valid address for reads up to `len` or up to the next null terminator
```

- `address: int` - the `WStr` pointer address.
- `**kwargs` - `null: boolean` - decode a string with a null terminator.
- `**kwargs` - `len: int` - decode a string with a specific length. this is not byte length. this is element length. that is, one element is a u16 (2 bytes).
- `**kwargs` - `lossy: boolean` - do not raise exception if provided string is not valid utf8. Warning, will return a string yes, but invalid characters will have been replaced with `�`.

## Using the type

To convert a `WStr` to a regular python string, just do `str(foo)` on your `WStr` type.

## Properties

### size: `int`
The byte size of the `WStr`. (The number of `u16` elements is `size / 2`)

### address: `int`
The address to the `WStr`'s buffer.

## Exceptions
If you supply an address and both null and len kwargs.

If you supply the address but neither null or len kwargs.

If the utf16 string to decode from an address is not a valid utf8 string (try using the lossy kwarg!).

If the first arg is not an address (`int`) or `str`.

If there is no first arg.

If you provide types to arg or kwargs that do not match the listed types.
