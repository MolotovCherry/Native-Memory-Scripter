# Object: IATSymbol

An import address table symbol.

## Drop
```admonish danger title=""
IAT entry will automatically unhook itself once deleted or garbage collected.
```

## Properties

#### name: Optional[str]
The name of the symbol.

#### ordinal: Optional[int]
The ordinal of the symbol.

#### dll_name: str
The name of the dll.

#### orig_fn: int
A pointer to the original iat entry function.

#### iat: int
A pointer to this IAT entry. Writing an address to this will hook it.

## Methods

### hook
Hook this iat entry.

```admonish danger title=""
This function is unsafe 🐉

- `address` must point to a `xr` function with the same signature as the original (abi, parameters, and return).
```

- `address: int` - the function address to redirect the iat entry to.

#### Exceptions
If virtual protect fails.

### unhook
Unhook this iat entry.

```admonish danger title=""
This function is unsafe 🐉
```

#### Exceptions
If virtual protect fails.
