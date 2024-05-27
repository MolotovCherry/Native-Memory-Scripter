# Function: get_thread_process

Gets a [`process`](./objects-process.md) from a [`thread`](./objects-thread.md). It is especially useful when you want to interact with a specific thread of a process.

```admonish success title=""
This function is safe
```

### Parameters
- <code>pthr: [thread](./objects-thread.md)</code> - [`thread`](./objects-thread.md) that will be used to find a [`process`](./objects-process.md).

### Return Value
On success, it returns [`process`](./objects-process.md); On failure, it returns `None`.
