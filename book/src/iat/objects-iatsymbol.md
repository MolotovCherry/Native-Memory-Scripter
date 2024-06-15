# Object: IATSymbol

An import address table symbol.

## Properties

#### name: Optional[str]
The name of the iat symbol.

#### ordinal: Optional[int]
The ordinal of the symbol.

#### dll_name: str
The name of the associated dll.

#### orig_fn: int
The address of the original function belonging to this iat entry.

#### iat
The address of the iat symbol where you can write to hook it.

## Methods

### hook

Hook this iat entry.

```admonish danger title=""
This function is unsafe 🐉

- `address` must point to a valid fn with correct abi, parameters, and return.
```

#### Exceptions
If changing protection status failed.

#### Parameters
- `address: int` - the address to change the iat entry to.

### unhook

Unhook this iat entry.

```admonish danger title=""
This function is unsafe 🐉
```

#### Exceptions
If changing protection status failed.
