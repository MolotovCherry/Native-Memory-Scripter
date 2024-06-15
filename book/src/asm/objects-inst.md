# Object: Inst

An assembly instruction

## Properties

#### address: int
If assembled with `address`, will contain the provided address offset this instruction will be at if written. Otherwise, the value here is not that meaningful. If diassembled, contains the starting address of the instruction

#### bytes: bytearray
the instructions binary data

#### mnemonic: str
the instruction's mnemonic

#### op_str: str
the instruction's operands
