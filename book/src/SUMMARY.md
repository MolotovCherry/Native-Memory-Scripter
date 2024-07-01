# Summary

[Introduction](./introduction.md)

# Project

- [Configuration](./configuration.md)

- [Project Layout and Setup](./project-layout-and-setup.md)

- [Best Practices](./best-practices.md)

- [Tips and Tricks](./tips-and-tricks.md)

- [Here be Dragons 🐉](./here-be-dragons.md)

- [FAQ](./faq.md)

# Modules

- [asm](./asm/asm.md)
    - [assemble](./asm/assemble.md)
    - [code_len](./asm/code_len.md)
    - [disassemble](./asm/disassemble.md)
    - [objects](./asm/objects.md)
        - [Inst](./asm/objects-inst.md)

- [cffi](./cffi/cffi.md)
    - [Type](./cffi/type.md)
    - [Conv](./cffi/conv.md)
    - [objects](./cffi/objects.md)
        - [Callable](./cffi/objects-callable.md)
        - [NativeCall](./cffi/objects-nativecall.md)
        - [WStr](./cffi/objects-wstr.md)

- [hook](./hook/hook.md)
    - [hook](./hook/hook_.md)
    - [objects](./hook/objects.md)
        - [Trampoline](./hook/objects-trampoline.md)

- [iat](./iat/iat.md)
    - [enum](./iat/enum.md)
    - [enum_demangled](./iat/enum_demangled.md)
    - [find](./iat/find.md)
    - [find_demangled](./iat/find_demangled.md)
    - [objects](./iat/objects.md)
        - [IATSymbol](./iat/objects-iatsymbol.md)

- [info](./info/info.md)
    - [version](./info/version.md)
    - [objects](./info/objects.md)
        - [Version](./info/object-version.md)

- [log](./log/log.md)
    - [trace](./log/trace.md)
    - [debug](./log/debug.md)
    - [info](./log/info.md)
    - [warn](./log/warn.md)
    - [error](./log/error.md)

- [mem](./mem/mem.md)
    - [alloc](./mem/alloc.md)
    - [alloc_in](./mem/alloc_in.md)
    - [deep_pointer](./mem/deep_pointer.md)
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

- [popup](./popup/popup.md)
    - [error](./popup/error.md)
    - [exclamation](./popup/exclamation.md)
    - [info](./popup/info.md)
    - [question](./popup/question.md)
    - [warn](./popup/warn.md)

- [scan](./scan/scan.md)
    - [data](./scan/data.md)
    - [pattern](./scan/pattern.md)
    - [sig](./scan/sig.md)

- [segments](./segments/segments.md)
    - [enum](./segments/enum.md)
    - [find](./segments/find.md)
    - [objects](./segments/objects.md)
        - [Segment](./segments/objects-segment.md)

- [symbols](./symbols/symbols.md)
    - [demangle](./symbols/demangle.md)
    - [enum](./symbols/enum.md)
    - [enum_demangled](./symbols/enum_demangled.md)
    - [find](./symbols/find.md)
    - [find_damangled](./symbols/find_demangled.md)

    - [objects](./symbols/objects.md)
        - [Symbol](./symbols/objects-symbol.md)

- [vmt](./vmt/vmt.md)
    - [objects](./vmt/objects.md)
        - [VTable](./vmt/objects-vtable.md)

# Final notes
- [Examples](./examples.md)
