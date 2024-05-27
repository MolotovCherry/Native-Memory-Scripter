# Function: get_page

Gets a [`page`](./objects-page.md) in the calling process from a virtual address.

```admonish success title=""
This function is safe
```

### Parameters
- `addr: int` - the virtual address that the page will be looked up from.

### Return Value
On success, it returns [`page`](./objects-page.md); On failure, it returns `None`.
