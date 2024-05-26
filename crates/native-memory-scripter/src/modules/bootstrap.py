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

# set in mem module
mem_attrs = [
    ("lm_prot_t", lm_prot_t)
]

# Set the attributes of mem
for (name, obj) in mem_attrs:
    setattr(mem, name, obj)
