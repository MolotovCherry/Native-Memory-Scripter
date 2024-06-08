use core::fmt;
use std::mem;

use crate::{
    asm::{self, AsmError},
    memory::{self, Alloc, MemError},
    Prot,
};

#[derive(Debug, thiserror::Error)]
pub enum HookError {
    #[error(transparent)]
    MemError(#[from] MemError),
    #[error(transparent)]
    AsmError(#[from] AsmError),
}

/// The trampoline to call the original function.
/// Once this type is dropped, it is _impossible_ to unhook and restore the original function back to normal!
/// Also, the trampoline code will be dropped and no longer be accessible, so you mustn't call the trampoline
/// if the memory was dropped.
#[derive(Debug)]
pub struct Trampoline {
    // the allocation holding the code - the code disappears when the trampoline is dropped!
    _code: Alloc,
    // the original ptr + length that was replaced
    from: (*mut u8, usize),
    pub address: *const u8,
    pub size: usize,
}

unsafe impl Send for Trampoline {}

impl fmt::Display for Trampoline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Trampoline {{ address: {:#x?}, size: {} }}",
            self.address, self.size
        )
    }
}

impl Trampoline {
    pub unsafe fn unhook(self) -> Result<(), HookError> {
        // remove memory protection
        let old = unsafe { memory::prot(self.from.0, self.from.1, Prot::XRW)? };

        // replace original fn code back to original location
        unsafe {
            memory::write_raw(self._code.as_ptr(), self.from.0, self.from.1);
        }

        // restore memory protection
        unsafe {
            memory::prot(self.from.0, self.from.1, old)?;
        }

        Ok(())
    }

    /// SAFETY: Caller must provide correct type signature
    pub unsafe fn callable<T: Copy>(&self) -> T {
        unsafe { mem::transmute_copy(&self._code.as_ptr::<T>()) }
    }
}

///
/// Starting at from address, finds next whole instruction and replaces it with
/// jmp to target address. The replaced instruction is placed inside the trampoline,
/// so caller must verify no relative instructions are replaced.
///
/// SAFETY:
///     - Must manually verify from location enough space for 14 bytes jmp to be written
///     - Must verify instruction that gets replaced is not relative
///     - Instruction that gets replaced should be able to ran in a different area of memory
///
pub unsafe fn hook(from: *mut u8, to: *const u8) -> Result<Trampoline, HookError> {
    debug_assert!(!from.is_null(), "from must not be null");
    debug_assert!(!to.is_null(), "to must not be null");

    // jmp code for trampoline
    #[rustfmt::skip]
    let mut jmp = [
        // jmp QWORD PTR [rip+0x0]
        0xFF, 0x25, 0x00, 0x00, 0x00, 0x00,
        // addr
        0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90,
    ];

    let code_len = unsafe { asm::code_len(from, jmp.len())? };

    //
    // generate the trampoline
    //

    // allocate some memory for our trampoline
    let trampoline_len = code_len + jmp.len();
    let trampoline = memory::alloc(trampoline_len, Prot::XRW)?;
    let trampoline_ptr = trampoline.as_ptr();
    // write original code to trampoline
    unsafe { memory::write_raw(from, trampoline_ptr, code_len) };

    // cause trampoline to call original function
    let target_addr = (from as usize + code_len).to_ne_bytes();
    // copy address into instruction array
    let si = jmp.len() - mem::size_of::<*const u8>();
    jmp[si..].copy_from_slice(&target_addr);

    // now write jmp
    unsafe { memory::write_bytes(&jmp, trampoline_ptr.add(code_len)) };

    // make it executable and readonly
    unsafe {
        memory::prot(trampoline_ptr, trampoline_len, Prot::XR)?;
    }

    //
    // copy the jmp to the original function to redirect it
    //

    // cause trampoline to call original function
    let target_addr = (to as usize).to_ne_bytes();
    // copy address into instruction array
    jmp[si..].copy_from_slice(&target_addr);

    // remove memory protection
    let prot_size = jmp.len();
    let old = unsafe { memory::prot(from, prot_size, Prot::XRW)? };

    // now write jmp
    unsafe {
        memory::write_bytes(&jmp, from);
    }

    // restore memory protection
    unsafe {
        memory::prot(from, prot_size, old)?;
    }

    //
    // end
    //

    let trampoline = Trampoline {
        from: (from, code_len),
        address: trampoline_ptr,
        _code: trampoline,
        size: trampoline_len,
    };

    Ok(trampoline)
}
