# Function: code_length

Gets the minimum instruction aligned length for `minlength` bytes from `code` in the calling process.

```admonish danger title=""
This function is unsafe 🐉
```

### Parameters
- `code: int` - Virtual address of the code to get the minimum aligned length from.
- `minlength: int` - The minimum length to align to an instruction length.

### Return Value
On success, it returns an `int` representing the minimum instruction aligned length for `minlength` bytes from `code`. On failure, it returns `None`.
