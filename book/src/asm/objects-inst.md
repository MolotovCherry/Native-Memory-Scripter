# Object: Inst

An assembly instruction

## Properties

#### address: int
If the `runtime_address` parameter was used, this will be the instruction's offset from that base address. Otherwise, the address starts at a default offset of `0`.

#### bytes: bytearray
The instructions binary data.

#### mnemonic: str
The instruction's mnemonic.

#### op_str: str
The instruction's operands.
