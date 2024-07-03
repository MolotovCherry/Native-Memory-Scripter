//! This module allows one to monitor reads and writes to memory ranges.

use core::slice;
use std::{
    cell::RefCell,
    cmp,
    collections::HashMap,
    mem,
    ops::Range,
    sync::{Arc, Mutex, Once, OnceLock},
};

use tracing::error;
use windows::Win32::{
    Foundation::{STATUS_ACCESS_VIOLATION, STATUS_GUARD_PAGE_VIOLATION, STATUS_SINGLE_STEP},
    System::{
        Diagnostics::Debug::{
            SetUnhandledExceptionFilter, CONTEXT, EXCEPTION_CONTINUE_EXECUTION,
            EXCEPTION_CONTINUE_SEARCH, EXCEPTION_POINTERS as WEXCEPTION_POINTERS, EXCEPTION_RECORD,
        },
        Memory::{
            VirtualProtect, PAGE_GUARD, PAGE_PROTECTION_FLAGS, PAGE_READONLY, PAGE_READWRITE,
        },
    },
};

use super::{prot, MemError};
use crate::{memory::get_page_size, utils::LazyLock, Prot};

#[derive(Debug, Copy, Clone)]
enum MonitorType {
    Read,
    Write,
}

impl MonitorType {
    unsafe fn start_monitor(&self, addr: *const (), size: usize) -> Result<Prot, MemError> {
        let _prot = match self {
            MonitorType::Read => PAGE_READWRITE | PAGE_GUARD,
            MonitorType::Write => PAGE_READONLY,
        };

        let mut old_prot = PAGE_PROTECTION_FLAGS::default();

        unsafe {
            VirtualProtect(addr.cast(), size, _prot, &mut old_prot)?;
        }

        Ok(old_prot.into())
    }

    unsafe fn stop_monitor(&self, addr: *const (), size: usize) -> Result<Prot, MemError> {
        unsafe { prot(addr, size, Prot::RW) }
    }
}

/// A guard that, when dropped, will undo the monitoring
#[derive(Debug)]
pub struct MonitorGuard {
    /// the base address this monitor guard belongs to. same address you passed in when you made this
    pub base: *const (),
    aligned_base: *const (),
    aligned_size: usize,
    ty: MonitorType,
}

unsafe impl Send for MonitorGuard {}
unsafe impl Sync for MonitorGuard {}

impl Drop for MonitorGuard {
    fn drop(&mut self) {
        unsafe {
            _ = self.ty.stop_monitor(self.aligned_base, self.aligned_size);
        }
    }
}

// so we can invoke the old exception handler if there was one
static OLD_EXC_HANDLER: OnceLock<
    unsafe extern "system" fn(exceptioninfo: *const WEXCEPTION_POINTERS) -> i32,
> = OnceLock::new();

#[allow(clippy::type_complexity)]
static MONITOR_CB: LazyLock<Mutex<HashMap<Range<usize>, MonitorCb>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

trait Overlapped {
    type Value;

    fn overlaps(&self, i: usize) -> bool;
    fn overlapping(&self, i: usize) -> Option<&Self::Value>;
}

impl<T> Overlapped for HashMap<Range<usize>, T> {
    type Value = T;

    fn overlaps(&self, i: usize) -> bool {
        for key in self.keys() {
            if key.contains(&i) {
                return true;
            }
        }

        false
    }

    fn overlapping(&self, i: usize) -> Option<&Self::Value> {
        self.iter().find(|(k, _)| k.contains(&i)).map(|(_, v)| v)
    }
}

#[derive(Clone)]
struct MonitorCb {
    aligned_base: *const (),
    aligned_size: usize,
    base: *const (),
    size: usize,
    ty: MonitorType,
    #[allow(clippy::type_complexity)]
    cb: Arc<dyn Fn(EXCEPTION_POINTERS, *const ()) + Send + Sync + 'static>,
}

unsafe impl Send for MonitorCb {}
unsafe impl Sync for MonitorCb {}

impl Eq for MonitorCb {}

impl cmp::PartialEq for MonitorCb {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.cb, &other.cb)
    }
}

/// Detects writes to memory address range and executes callback with info.
/// This callback will be executed before the write happens.
///
/// Note that this will delay execution of the program. Make the callback speedy.
///
/// # Safety
/// The base address must be valid for base + page size (or multiple page sizes of size_of::<T>() > page_size), and must not be executable memory
/// Memory must be readable + writeable
/// Do not use this with stack memory
/// Size to take care of depends on size of T. Make sure you put a T of the right size there!
pub unsafe fn monitor_writes<T>(
    base: *const T,
    f: impl Fn(EXCEPTION_POINTERS, *const ()) + Send + Sync + 'static,
) -> Result<MonitorGuard, MemError> {
    unsafe { monitor(base, MonitorType::Write, f) }
}

/// Detects reads to memory address range and executes callback with info.
/// Due to api limitations, the callback will be executed after the read happens.
///
/// Note that this will delay execution of the program. Make the callback speedy.
///
/// # Safety
/// The base address must be valid for base + page size (or multiple page sizes of size_of::<T>() > page_size), and must not be executable memory
/// Memory must be readable + writeable
/// Do not use this with stack memory
/// Size to take care of depends on size of T. Make sure you put a T of the right size there!
pub unsafe fn monitor_reads<T>(
    base: *const T,
    f: impl Fn(EXCEPTION_POINTERS, *const ()) + Send + Sync + 'static,
) -> Result<MonitorGuard, MemError> {
    unsafe { monitor(base, MonitorType::Read, f) }
}

unsafe fn monitor<T>(
    base: *const T,
    ty: MonitorType,
    f: impl Fn(EXCEPTION_POINTERS, *const ()) + Send + Sync + 'static,
) -> Result<MonitorGuard, MemError> {
    static INSTALL: Once = Once::new();
    INSTALL.call_once(|| {
        let old_handler = unsafe { SetUnhandledExceptionFilter(Some(exception_handler)) };
        if let Some(handler) = old_handler {
            // save old handler so we can call it later
            _ = OLD_EXC_HANDLER.set(handler);
        }
    });

    let base = base as *const ();
    let page_size = get_page_size() as usize;
    let size = mem::size_of::<T>();

    let aligned_base = (base as usize & !(page_size - 1)) as *const ();
    let aligned_size =
        ((base as usize + size - 1 + page_size) & !(page_size - 1)) - aligned_base as usize;

    let range = Range {
        start: aligned_base as usize,
        end: aligned_base as usize + aligned_size,
    };

    let mut cb = MONITOR_CB.lock().unwrap();
    if cb.overlaps(aligned_base as usize) {
        return Err(MemError::Overlaps);
    }

    unsafe {
        ty.start_monitor(aligned_base, aligned_size)?;
    }

    let mon_cb = MonitorCb {
        aligned_base,
        aligned_size,
        base,
        size,
        cb: Arc::new(f),
        ty,
    };

    cb.insert(range, mon_cb);

    let guard = MonitorGuard {
        base,
        aligned_base,
        aligned_size,
        ty,
    };

    Ok(guard)
}

unsafe extern "system" fn exception_handler(raw_exc: *const WEXCEPTION_POINTERS) -> i32 {
    // handle using the old handler
    let handle = || {
        OLD_EXC_HANDLER
            .get()
            .map(|f| unsafe { f(raw_exc) })
            .unwrap_or(EXCEPTION_CONTINUE_SEARCH)
    };

    let exc = unsafe { &*raw_exc };

    let ctx = unsafe { &mut *exc.ContextRecord };
    let record = unsafe { &*exc.ExceptionRecord };
    let info = record.ExceptionInformation;

    let addr = info[1];

    thread_local! {
        static STEP_QUEUE: RefCell<Option<MonitorCb>> = const { RefCell::new(None) };
        // for tracking read only state
        static STEP_QUEUE_VAL: RefCell<(EXCEPTION_POINTERS, Vec<u8>)> = RefCell::new((EXCEPTION_POINTERS::default(), Vec::with_capacity(get_page_size() as usize)));
    }

    match record.ExceptionCode {
        STATUS_ACCESS_VIOLATION if info[0] == 1 => {
            let cb = MONITOR_CB.lock().unwrap();

            let monitor_cb = cb.overlapping(addr);

            if let Some(&MonitorCb {
                base,
                size,
                aligned_base,
                aligned_size,
                cb: ref f,
                ty,
            }) = monitor_cb
            {
                let res = unsafe { ty.stop_monitor(aligned_base, aligned_size) };
                if res.is_err() {
                    // we can't do anything else here if it failed except continue the exception
                    return handle();
                }

                if addr >= base as usize && addr < (base as usize + size) {
                    f(exc.into(), base);
                }

                STEP_QUEUE.with_borrow_mut(|cb| {
                    *cb = monitor_cb.cloned();
                });

                // next instruction, which is the instruction that has caused
                // this page fault (AKA access violation)
                ctx.EFlags |= 1 << 8;

                EXCEPTION_CONTINUE_EXECUTION
            } else {
                handle()
            }
        }

        STATUS_GUARD_PAGE_VIOLATION => {
            let cb = MONITOR_CB.lock().unwrap();

            let monitor_cb = cb.overlapping(addr);

            if let Some(&MonitorCb { base, size, .. }) = monitor_cb {
                if addr >= base as usize && addr < (base as usize + size) {
                    STEP_QUEUE_VAL.with_borrow_mut(|step_data| {
                        // record exception so we can use it in the callback on next step
                        step_data.0 = exc.into();

                        let data = unsafe { slice::from_raw_parts(base.cast::<u8>(), size) };
                        step_data.1.extend_from_slice(data);
                    });
                }

                STEP_QUEUE.with_borrow_mut(|cb| {
                    *cb = monitor_cb.cloned();
                });

                // next instruction, which is the instruction that has caused
                // this page fault (AKA access violation)
                ctx.EFlags |= 1 << 8;

                EXCEPTION_CONTINUE_EXECUTION
            } else {
                handle()
            }
        }

        STATUS_SINGLE_STEP => {
            STEP_QUEUE.with_borrow_mut(|cb| {
                if let Some(cb) = cb.take() {
                    if matches!(cb.ty, MonitorType::Read) {
                        let data = unsafe { slice::from_raw_parts(cb.base.cast::<u8>(), cb.size) };
                        STEP_QUEUE_VAL.with_borrow_mut(|val| {
                            let new_data = &val.1[..cb.size];

                            if data == new_data {
                                (cb.cb)(val.0, cb.base);
                            }

                            val.1.clear();
                        });
                    }

                    // we read data above. in the case of live reads, this will redo the guard page, so we need this to be after the read
                    let res = unsafe { cb.ty.start_monitor(cb.aligned_base, cb.aligned_size) };
                    if let Err(e) = res {
                        error!("monitor failed to set page back to monitor status: {e}");
                    }
                } else {
                    error!(
                        "failed to reset page to monitor status. This is a bug, please report it."
                    );
                }
            });

            EXCEPTION_CONTINUE_EXECUTION
        }

        _ => handle(),
    }
}

/// https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-exception_pointers
#[allow(
    non_camel_case_types,
    non_snake_case,
    missing_debug_implementations,
    missing_docs
)]
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct EXCEPTION_POINTERS {
    pub ExceptionRecord: EXCEPTION_RECORD,
    pub ContextRecord: CONTEXT,
}

unsafe impl Send for EXCEPTION_POINTERS {}
unsafe impl Sync for EXCEPTION_POINTERS {}

impl From<WEXCEPTION_POINTERS> for EXCEPTION_POINTERS {
    fn from(exc: WEXCEPTION_POINTERS) -> Self {
        Self {
            ExceptionRecord: unsafe { *exc.ExceptionRecord },
            ContextRecord: unsafe { *exc.ContextRecord },
        }
    }
}

impl From<&WEXCEPTION_POINTERS> for EXCEPTION_POINTERS {
    fn from(exc: &WEXCEPTION_POINTERS) -> Self {
        (*exc).into()
    }
}
