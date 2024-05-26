import mem
from enum import IntEnum

class lm_prot_t(IntEnum):
    '''Protection flags'''
    NONE = 0b000
    # Executable
    X = 0b001
    # Read
    R = 0b010
    # Write
    W = 0b100
    # Executable + Read
    XR = 0b011
    # Executable + Write
    XW = 0b101
    # Read + Write
    RW = 0b110
    # Executable + Read + Write
    XRW = 0b111

# Get the current global symbol table
g = dict(globals())

skip = ["mem", "IntEnum"]

# Set the attributes of mem
for name, obj in g.items():
    if name not in skip:
        setattr(mem, name, obj)
