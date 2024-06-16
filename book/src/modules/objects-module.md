# Object: Module

A module.

## Drop
```admonish note title=""
Module will unload once gc collects this.

An unloaded module does not necessarily mean it's unloaded from memory. Windows holds a refcount, and when free library is called, the refcount decreases by 1. This handle increases refcount by 1 when it's made so it's always safe to use as long as the handle is alive, thus it's safe to decrease it by 1.
```

## Properties

#### base: int
The base address of the module.

#### end: int
The end address of the module.

#### size: int
The size of the module.

#### path: str
The full path to the module.

#### name: str
The name of the module, e.g. `foobar.dll`.
