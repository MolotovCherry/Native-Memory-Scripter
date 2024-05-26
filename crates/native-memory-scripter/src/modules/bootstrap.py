
from enum import IntEnum

#
# Start mem
#

import mem

# Formally `lm_prot_t`
class Prot(IntEnum):
    '''Protection flags'''
    # None
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

    def __repr__(self):
        return self.name

    def __str__(self):
        return self.name

# set in mem module
mem_attrs = [
    ("Prot", Prot)
]

# Set the attributes of mem
for (name, obj) in mem_attrs:
    setattr(mem, name, obj)

#
# End mem
#
