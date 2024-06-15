# Summary

[Introduction](./introduction.md)

# Project

- [Configuration](./configuration.md)

- [Project Layout and Setup](./project-layout-and-setup.md)

- [Best Practices](./best-practices.md)

- [Here be Dragons 🐉](./here-be-dragons.md)

# Modules

- [asm](./asm/asm.md)
    - [assemble](./asm/assemble.md)
    - [assemble_ex](./asm/assemble_ex.md)
    - [code_len](./asm/code_len.md)
    - [disassemble](./asm/disassemble.md)
    - [objects](./asm/objects.md)
        - [Inst](./asm/objects-inst.md)

- [vtable](./vtable/vtable.md)
    - [objects](./vtable/objects.md)
        - [VTable](./vtable/objects-vtable.md)

- [iat](./iat/iat.md)
    - [enum](./iat/enum.md)
    - [enum_demangled](./iat/enum_demangled.md)
    - [find](./iat/find.md)
    - [find_ordinal](./iat/find_ordinal.md)
    - [find_with_dll](./iat/find_with_dll.md)
    - [find_with_dll_ordinal](./iat/find_with_dll_ordinal.md)
    - [find_demangled](./iat/find_demangled.md)
    - [find_with_dll_demangled](./iat/find_with_dll_demangled.md)
    - [objects](./iat/objects.md)
        - [IATSymbol](./iat/objects-iatsymbol.md)

- [mem](./mem/mem.md)
    - [alloc](./mem/alloc.md)
    - [read](./mem/read.md)
    - [set](./mem/set.md)
    - [write](./mem/write.md)
    - [prot](./mem/prot.md)
    - [objects](./mem/objects.md)
        - [Alloc](./mem/objects-alloc.md)
        - [Prot](./mem/objects-prot.md)

- [modules](./modules/modules.md)
    - [load](./modules/load.md)
    - [unload](./modules/unload.md)
    - [find](./modules/find.md)
    - [enum](./modules/enum.md)
    - [objects](./modules/objects.md)
        - [Module](./modules/objects-module.md)

- [symbols](./symbols/symbols.md)
    - [demangle](./symbols/demangle.md)
    - [enum](./symbols/enum.md)
    - [enum_demangled](./symbols/enum_demangled.md)
    - [find_address](./symbols/find_address.md)
    - [find_address_damangled](./symbols/find_address_demangled.md)

    - [objects](./symbols/objects.md)
        - [Symbol](./symbols/objects-symbol.md)

- [segments](./segments/segments.md)
    - [enum](./segments/enum.md)
    - [find](./segments/find.md)
    - [objects](./segments/objects.md)
        - [Segment](./segments/objects-segment.md)

- [scan](./scan/scan.md)
    - [data_scan](./scan/data_scan.md)
    - [pattern_scan](./scan/pattern_scan.md)
    - [sig_scan](./scan/sig_scan.md)

- [info](./info/info.md)
    - [version](./info/version.md)
    - [objects](./info/objects.md)
        - [Version](./info/object-version.md)

# Final notes
- [Examples](./examples.md)
