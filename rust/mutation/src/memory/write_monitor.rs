//! This module allows one to monitor writes to memory ranges.

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
    Foundation::{STATUS_ACCESS_VIOLATION, STATUS_SINGLE_STEP},
    System::Diagnostics::Debug::{
        SetUnhandledExceptionFilter, EXCEPTION_CONTINUE_EXECUTION, EXCEPTION_CONTINUE_SEARCH,
        EXCEPTION_POINTERS,
    },
};

use super::{prot, MemError};
use crate::{memory::get_page_size, utils::LazyLock, Prot};

/// The Id for write monitor
pub type MonitorId = usize;

/// A guard that, when dropped, will undo the monitoring
#[derive(Debug)]
pub struct MonitorGuard {
    /// the base address this monitor guard belongs to. same address you passed in when you made this
    pub base: *const (),
    aligned_base: *const (),
    aligned_size: usize,
    old_prot: Prot,
}

unsafe impl Send for MonitorGuard {}
unsafe impl Sync for MonitorGuard {}

impl Drop for MonitorGuard {
    fn drop(&mut self) {
        _ = unsafe { prot(self.aligned_base, self.aligned_size, self.old_prot) };
    }
}

// so we can invoke the old exception handler if there was one
static OLD_EXC_HANDLER: OnceLock<
    unsafe extern "system" fn(exceptioninfo: *const EXCEPTION_POINTERS) -> i32,
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
    #[allow(clippy::type_complexity)]
    cb: Arc<dyn Fn(&EXCEPTION_POINTERS, *const ()) + Send + Sync + 'static>,
}

unsafe impl Send for MonitorCb {}
unsafe impl Sync for MonitorCb {}

impl Eq for MonitorCb {}

impl cmp::PartialEq for MonitorCb {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.cb, &other.cb)
    }
}

/// Detects writes to memory address range and executes callback with info
///
/// # Safety
/// The base address must be valid for base + page size (or multiple page sizes of size_of::<T>() > page_size), and must not be executable memory
/// Memory must be readable + writeable
/// Do not use this with stack memory
/// Size to take care of depends on size of T. Make sure you put a T of the right size there!
pub unsafe fn monitor_writes<T>(
    base: *const T,
    f: impl Fn(&EXCEPTION_POINTERS, *const ()) + Send + Sync + 'static,
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

    let old_prot = unsafe { prot(aligned_base, aligned_size, Prot::R)? };

    let mon_cb = MonitorCb {
        aligned_base,
        aligned_size,
        base,
        size,
        cb: Arc::new(f),
    };

    cb.insert(range, mon_cb);

    let guard = MonitorGuard {
        base,
        aligned_base,
        aligned_size,
        old_prot,
    };

    Ok(guard)
}

unsafe extern "system" fn exception_handler(raw_exc: *const EXCEPTION_POINTERS) -> i32 {
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
                ..
            }) = monitor_cb
            {
                let res = unsafe { prot(aligned_base, aligned_size, Prot::RW) };
                if res.is_err() {
                    // we can't do anything else here if it failed except continue the exception
                    return handle();
                }

                if addr >= base as usize && addr < (base as usize + size) {
                    f(exc, base);
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
                    _ = unsafe { prot(cb.aligned_base, cb.aligned_size, Prot::R) };
                } else {
                    error!("failed to reset page to Prot::R. This is a bug, please report it.");
                }
            });

            EXCEPTION_CONTINUE_EXECUTION
        }

        _ => handle(),
    }
}
