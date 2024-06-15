# Function: code_len

Given a starting address and minimum length, finds the closest byte length that form valid instructions

```admonish danger title=""
This function is unsafe 🐉

- The `code` address must at minimum be valid for reads from `min_length` up to the nearest instruction end
```

### Parameters
- `code: int` - the starting address of the code
- `min_length: int` - the minimum desired byte length you want

### Exceptions
If it fails to diassemble

### Return Value
Returns an `int` representing the length in bytes of the nearest valid instructions to the `min_length`
