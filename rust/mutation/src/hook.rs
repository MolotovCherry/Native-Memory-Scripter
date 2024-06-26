//! This module allows one to hook functions

use std::{
    fmt, mem, ptr,
    sync::{Arc, Mutex},
};

use arrayvec::ArrayVec;
use tracing::trace;

use crate::{
    asm::{self, AsmError},
    memory::{self, Alloc, MemError},
    Prot,
};

/// An error for the [hook](crate::hook) module
#[derive(Debug, thiserror::Error)]
pub enum HookError {
    /// a memory error
    #[error(transparent)]
    MemError(#[from] MemError),
    /// an asm error
    #[error(transparent)]
    AsmError(#[from] AsmError),
}

/// The trampoline to call the original function.
///
/// Once this type is dropped, it will automatically unhook itself!
/// Also, the trampoline code will be dropped and no longer be accessible, so you mustn't call the trampoline
/// if the memory was dropped.
pub struct Trampoline {
    // the allocation holding the code - the code disappears when the trampoline is dropped!
    _code: Arc<Alloc>,
    // the original ptr + length that was replaced
    from: (*mut u8, usize),
    mutex: Mutex<()>,
    /// the trampoline address
    pub address: *const u8,
    /// the code size of the trampoline
    pub size: usize,
}

impl fmt::Debug for Trampoline {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Trampoline { address, size, .. } = self;

        f.debug_struct("Trampoline")
            .field("address", &address)
            .field("size", &size)
            .finish()
    }
}

unsafe impl Send for Trampoline {}
unsafe impl Sync for Trampoline {}

impl Clone for Trampoline {
    fn clone(&self) -> Self {
        Self {
            _code: self._code.clone(),
            from: self.from,
            mutex: Mutex::default(),
            address: self.address,
            size: self.size,
        }
    }
}

impl fmt::Display for Trampoline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Trampoline {{ address: {:?}, size: {} }}",
            self.address, self.size
        )
    }
}

impl Trampoline {
    /// Unhook the trampoline and restore original operation of the hooked function
    ///
    /// # Safety
    /// This overwrites the target function with the original code. There is no synchronization.
    pub unsafe fn unhook(&self) -> Result<(), HookError> {
        unsafe { self._unhook() }
    }

    /// Create a callable function to the trampoline
    ///
    /// ```rust,ignore
    /// let t = hook.callable::<fn()>();
    /// t();
    /// ```
    ///
    /// # Safety
    /// Caller must provide correct type signature
    pub unsafe fn callable<T: Copy>(&self) -> T {
        unsafe { mem::transmute_copy(&self._code.addr()) }
    }

    unsafe fn _unhook(&self) -> Result<(), HookError> {
        let _guard = self.mutex.lock().unwrap();

        trace!(
            "unhook copying {} bytes from {:?} -> {:?}",
            self.from.1,
            self._code.addr(),
            self.from.0
        );

        // remove memory protection
        let old = unsafe { memory::prot(self.from.0.cast(), self.from.1, Prot::XRW)? };

        // replace original fn code back to original location
        unsafe {
            memory::write_raw(self._code.addr() as _, self.from.0, self.from.1);
        }

        // restore memory protection
        unsafe {
            memory::prot(self.from.0.cast(), self.from.1, old)?;
        }

        Ok(())
    }
}

impl Drop for Trampoline {
    fn drop(&mut self) {
        _ = unsafe { self._unhook() };
    }
}

fn make_jmp(from: *mut u8, to: *const u8, force_64: bool) -> ArrayVec<u8, 14> {
    let mut jmp = ArrayVec::<_, 14>::new();

    // jmp code for trampoline
    #[rustfmt::skip]
    let mut jmp64 = [
        // jmp [rip]
        0xFF, 0x25, 0x00, 0x00, 0x00, 0x00,
        // addr
        0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90,
    ];

    let mut jmp32 = [0xE9, 0x0, 0x0, 0x0, 0x0]; // jmp <addr>

    let relative_addr: Option<i32> = (to as isize)
        .checked_sub(from as isize)
        .and_then(|i| i.checked_sub(jmp32.len() as isize))
        .and_then(|n| n.try_into().ok());

    if relative_addr.is_none() || force_64 {
        jmp64[6..].copy_from_slice(&(to as usize).to_ne_bytes());
        jmp.try_extend_from_slice(&jmp64).unwrap();
    } else if let Some(addr) = relative_addr {
        jmp32[1..].copy_from_slice(&addr.to_ne_bytes());
        jmp.try_extend_from_slice(&jmp32).unwrap();
    }

    jmp
}

/// Starting at from address, finds next whole instruction and replaces it with
/// jmp to target address. The replaced instruction is placed inside the trampoline,
/// so caller must verify no relative instructions are replaced, as these are not
/// valid if they're in another location.
///
/// If `to` address is within 32-bits of `from`, uses relative 32-bit jmp (5 bytes), otherwise
/// will take 14 bytes for a full 64-bit jmp
///
/// # Safety
/// - Must manually verify `from`` location enough space for 14 or 5 bytes jmp to be written
/// - Must verify instruction that gets replaced is not relative
/// - Instruction that gets replaced should be able to ran in a different area of memory
pub unsafe fn hook(from: *mut u8, to: *const u8) -> Result<Trampoline, HookError> {
    debug_assert!(!from.is_null(), "from must not be null");
    debug_assert!(!to.is_null(), "to must not be null");

    //
    // copy the jmp to the original function to redirect it
    //

    // generate 5 or 14 byte jmp, whichever is possible
    let jmp = make_jmp(from, to, false);

    // we will need these later for the trampoline
    let code_len = unsafe { asm::code_len(from, jmp.len())? };
    let orig_bytes = unsafe { memory::read_bytes(from, code_len) };

    trace!(
        "jmp -> {to:?} used {} bytes spanning {from:?}-0x{:x}",
        jmp.len(),
        from as usize + (code_len.saturating_sub(1))
    );

    // remove memory protection
    let prot_size = jmp.len();
    let old = unsafe { memory::prot(from.cast(), prot_size, Prot::XRW)? };

    // now write jmp
    unsafe {
        memory::write_bytes(&jmp, from);
    }

    // restore memory protection
    unsafe {
        memory::prot(from.cast(), prot_size, old)?;
    }

    //
    // generate the trampoline
    //

    // generate full 64-bit jmp for trampoline
    // when force is on, `from` addr is not used
    let target = unsafe { from.add(code_len) };
    let jmp = make_jmp(ptr::null_mut(), target, true);

    // allocate some memory for our trampoline
    let trampoline_len = orig_bytes.len() + jmp.len();
    let trampoline = memory::alloc(trampoline_len, Prot::XRW)?;

    trace!("trampoline @ {:?} jmp -> {:?}", trampoline.addr(), target);

    // write original code to trampoline
    unsafe { memory::write_bytes(&orig_bytes, trampoline.addr()) };

    // now write jmp
    unsafe { memory::write_bytes(&jmp, trampoline.addr().add(orig_bytes.len())) };

    // make it executable and readonly
    unsafe {
        memory::prot(trampoline.addr().cast(), trampoline_len, Prot::XR)?;
    }

    //
    // end
    //

    let trampoline = Trampoline {
        from: (from, code_len),
        address: trampoline.addr(),
        _code: Arc::new(trampoline),
        size: trampoline_len,
        mutex: Mutex::default(),
    };

    Ok(trampoline)
}
