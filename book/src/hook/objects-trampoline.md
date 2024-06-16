# Object: Trampoline

An import address table symbol.

## Drop
```admonish note title=""
Trampoline will free the underlying trampoline code, and unhook the target.
```

## Properties

#### address: int
The address of the trampoline function.

#### size: int
The size of the trampoline function in bytes.

## Methods

### unhook
Unhook the hooked function.

```admonish danger title=""
This function is unsafe 🐉
```

#### Exceptions
If virtual protect fails.
