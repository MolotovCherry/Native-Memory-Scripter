# Function: find

Find a module by name.

```admonish success title=""
This function is safe
```

### Parameters
- `name: str` - the name to search for. exact case-sensitive match, including extension. e.g. `foobar.dll`.

### Exceptions
If unable to create a snapshot, no modules exist, unable to convert from utf16 to utf8, or load library.

### Return Value
Returns a [`Module`](./objects-module.md) if found, `None` if not found.
