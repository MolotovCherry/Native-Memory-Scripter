# VTable

Hook a c++ vtable.

## Constructor

- `address: int` - the base address of the vtable.

## Drop

```admonish info title=""
When this is deleted, it will automatically reset all hooks to default.
```

## Methods

### hook
Hooks a vtable by index.

```admonish danger title=""
This function is unsafe 🐉

- `index` must be a valid vtable index.
- `dst` must point to a `xr` function with the same signature as the original (abi, parameters, and return).
```

- `index: int` - the index to hook.
- `dst: int` - the function address to redirect the vtable entry to.

#### Exceptions
If virtual protect fails.

### unhook
Unhooks a vtable by index.

```admonish danger title=""
This function is unsafe 🐉

- `index` must be a valid previously hooked vtable index.
```

- `index: int` - the index to unhook.

#### Exceptions
If virtual protect fails.

### get_original
Get a pointer to the original function stored at a previously hooked index.

```admonish success title=""
This function is safe
```

- `index: int` - the index to unhook.

#### Return Value
Returns an `int` representing a ptr to the original vtable function if index was previously hooked, otherwise returns `None`.

### reset
Resets all previously hooked vtable entries back to their original functions.

```admonish danger title=""
This function is unsafe 🐉
```

#### Exceptions
If virtual protect fails.
