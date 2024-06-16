# Tips and Tricks

~~~admonish tip title = "Use the `struct` module" collapsible = true
Use the python `struct` module to pack and unpack custom structs over ffi.

Let's say you received a ptr to a struct:
```rust
#[repr(C)]
struct Foo {
    foo: u32,
    bar: u64,
    baz: *const () // platform sized ptr
}
```
Then you could pack or unpack it in python like so
```python
import struct

class Foo:
    def __init__(self, bar, baz, some_ptr):
        self.bar = bar
        self.baz = baz
        self.some_ptr = some_ptr

    def pack(self):
        return struct.pack('IQP', self.bar, self.baz, self.some_ptr)

    @classmethod
    def unpack(cls, packed):
        bar, baz, some_ptr = struct.unpack('IQP', packed)
        return cls(bar, baz, some_ptr)
```
~~~
