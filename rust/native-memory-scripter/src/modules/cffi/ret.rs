use std::{
    alloc::{self, Layout},
    ffi,
};

use rustpython_vm::{
    builtins::PyBytes,
    convert::ToPyObject,
    prelude::{PyObjectRef, VirtualMachine},
    PyResult,
};
use tracing::warn;

use super::{cffi::WStr, types::Type};
use crate::utils::RawSendable;

#[allow(improper_ctypes_definitions)]
#[repr(C)]
pub union Ret {
    pub f32: f32,
    pub f64: f64,

    pub u8: u8,
    pub u16: u16,
    pub u32: u32,
    pub u64: u64,
    pub u128: u128,

    pub i8: i8,
    pub i16: i16,
    pub i32: i32,
    pub i64: i64,
    pub i128: i128,

    pub bool: bool,

    pub ptr: usize,

    pub char: u8,
    pub wchar: u16,
}

impl Ret {
    // Python fn return -> callback return
    pub fn write_ret(
        val: PyObjectRef,
        ty: Type,
        ret: *mut Ret,
        vm: &VirtualMachine,
    ) -> PyResult<()> {
        #[rustfmt::skip]
        let data =
            match ty {
                Type::Void => {
                    // sret is null
                    return Ok(());
                }

                Type::F32(_) => {
                    let f = val.try_into_value(vm)?;
                    Ret { f32: f }
                }

                Type::F64(_) => {
                    let f = val.try_into_value(vm)?;
                    Ret { f64: f }
                }

                Type::U8(_) => {
                    let u = val.try_into_value(vm)?;
                    Ret { u8: u }
                }

                Type::U16(_) => {
                    let u = val.try_into_value(vm)?;
                    Ret { u16: u }
                }

                Type::U32(_) => {
                    let u = val.try_into_value(vm)?;
                    Ret { u32: u }
                }

                Type::U64(_) => {
                    let u = val.try_into_value(vm)?;
                    Ret { u64: u }
                }

                Type::U128(_) => {
                    let u = val.try_into_value(vm)?;
                    Ret { u128: u }
                }

                Type::I8(_) => {
                    let i = val.try_into_value(vm)?;
                    Ret { i8: i }
                }

                Type::I16(_) => {
                    let i = val.try_into_value(vm)?;
                    Ret { i16: i }
                }

                Type::I32(_) => {
                    let i = val.try_into_value(vm)?;
                    Ret { i32: i }
                }

                Type::I64(_) => {
                    let i = val.try_into_value(vm)?;
                    Ret { i64: i }
                }

                Type::I128(_) => {
                    let i = val.try_into_value(vm)?;
                    Ret { i128: i }
                }

                Type::Ptr(_) => {
                    let ptr = val.try_into_value(vm)?;
                    Ret { ptr }
                }

                Type::Bool(_) => {
                    let b = val.try_into_value(vm)?;
                    Ret { bool: b }
                }

                // User returns a regular string.
                //
                // # Safety
                // - It must be null terminated
                // - The lifetime of the python string object must be >= how long it will be used in C
                Type::CStr(_) => {
                    let pystr = val.str(vm)?;

                    let pystr = pystr.as_str();

                    if !pystr.ends_with('\0') {
                        // return empty string with null byte.. At least it's safer than UB
                        warn!("returned cstr does not end with null byte; this will cause UB, so empty cstr returned instead");

                        let null = "\0";

                        Ret {
                            ptr: null.as_ptr() as _,
                        }
                    } else {
                        let ptr = pystr.as_ptr() as usize;

                        Ret { ptr }
                    }
                }

                // User returns a utf16 str.
                //
                // SAFETY:
                // - If api requires it, it must be null terminated
                // - If api requires it, it must be correct length you specified to api
                // - The lifetime of the python string object must be >= how long it will be used in C
                Type::WStr(_) => {
                    let wstr = val.downcast_ref::<WStr>().ok_or_else(|| {
                        vm.new_type_error("failed to convert to WStr".to_owned())
                    })?;

                    let ptr = wstr.as_ptr();

                    Ret { ptr: ptr as usize }
                }

                // User returns a str with a single character
                //
                // User must provide a string with exactly 1 char long.
                //
                // Note: a single char means "a character that can be represented as a u8".
                //       therefore "a single char" is not utf8.
                //       e.g, "á" looks like a single char but it's actually 2 bytes long
                Type::Char(_) => {
                    let c = val.try_into_value::<String>(vm)?;

                    if c.len() != 1 {
                        return Err(vm.new_runtime_error(format!(
                            "Char expected byte len of 1, instead got {}",
                            c.len()
                        )));
                    }

                    let c = c
                        .chars()
                        .next()
                        .map(|c| c.try_into().unwrap())
                        .ok_or_else(|| vm.new_type_error("expected char".to_owned()))?;

                    Ret { char: c }
                }

                // User returns a str with a single character
                //
                // User must provide a string with exactly 1 char long.
                //
                // Note: a single char means "a character that can be represented as a u16".
                //       therefore "a single char" is not utf8.
                //       e.g, "你" looks like a single char but it's actually 3 bytes long
                Type::WChar(_) => {
                    let c = val.try_into_value::<String>(vm)?;

                    if c.len() <= 1 {
                        return Err(vm.new_runtime_error(format!(
                            "WChar expected byte len of <= 2, instead got {}",
                            c.len()
                        )));
                    }

                    let c = c
                        .chars()
                        .next()
                        .map(|c| c.try_into().unwrap())
                        .ok_or_else(|| vm.new_type_error("expected wchar".to_owned()))?;

                    Ret { wchar: c }
                }

                // User returns
                //
                // User returns from function with bytes
                //
                // # Safety
                // User asserts size is correct, and asserts their data is a valid T
                Type::Struct(size) => {
                    let bytes = val.bytes(vm)?.downcast::<PyBytes>().map_err(|_| {
                        vm.new_type_error("failed to convert to PyBytes".to_owned())
                    })?;

                    if bytes.len() != size as usize {
                        return Err(vm.new_runtime_error(format!(
                            "Struct return expected len {size}, instead got {}",
                            bytes.len()
                        )));
                    }

                    let bytes = bytes.as_bytes();

                    match size {
                        1 => unsafe {
                            *ret = Ret { i8: bytes.as_ptr().cast::<i8>().read() };
                        },

                        2 => unsafe {
                            let bytes = bytes.as_ptr().cast::<i16>().read_unaligned();
                            *ret = Ret { i16: bytes };
                        },

                        4 => unsafe {
                            let bytes = bytes.as_ptr().cast::<i32>().read_unaligned();
                            *ret = Ret { i32: bytes };
                        },

                        8 => unsafe {
                            let bytes = bytes.as_ptr().cast::<i64>().read_unaligned();
                            *ret = Ret { i64: bytes };
                        },

                        // it's a ptr!
                        _ => unsafe {
                            mem::memory::write_bytes(bytes, ret.cast());
                        },
                    }

                    // there's no output since we write directly to it
                    return Ok(());
                }
            };

        // write the data
        // if is void, ptr is null, otherwise it's not
        if !ty.is_void() {
            unsafe {
                *ret = data;
            }
        }

        Ok(())
    }

    /// SAFETY:
    /// Caller asserts that Type represents the type held in Ret
    pub unsafe fn to_pyobject(&self, ret: Type, vm: &VirtualMachine) -> PyObjectRef {
        match ret {
            Type::Void => None::<()>.to_pyobject(vm),
            Type::F32(_) => unsafe { self.f32.to_pyobject(vm) },
            Type::F64(_) => unsafe { self.f64.to_pyobject(vm) },
            Type::U8(_) => unsafe { self.u8.to_pyobject(vm) },
            Type::U16(_) => unsafe { self.u16.to_pyobject(vm) },
            Type::U32(_) => unsafe { self.u32.to_pyobject(vm) },
            Type::U64(_) => unsafe { self.u64.to_pyobject(vm) },
            Type::U128(_) => unsafe { self.u128.to_pyobject(vm) },
            Type::I8(_) => unsafe { self.i8.to_pyobject(vm) },
            Type::I16(_) => unsafe { self.i16.to_pyobject(vm) },
            Type::I32(_) => unsafe { self.i32.to_pyobject(vm) },
            Type::I64(_) => unsafe { self.i64.to_pyobject(vm) },
            Type::I128(_) => unsafe { self.i128.to_pyobject(vm) },
            Type::Ptr(_) => unsafe { self.ptr.to_pyobject(vm) },
            Type::Bool(_) => unsafe { self.bool.to_pyobject(vm) },

            // null terminated
            Type::CStr(_) => {
                let ptr = self.ptr as *const i8;

                if ptr.is_null() {
                    return None::<()>.to_pyobject(vm);
                }

                let data = unsafe { ffi::CStr::from_ptr(ptr) };
                let string = data.to_string_lossy();
                string.to_pyobject(vm)
            }

            // wstr is not required to terminate with null. we don't know the length, so we can only return the ptr
            Type::WStr(_) => unsafe { self.ptr.to_pyobject(vm) },

            Type::Char(_) => {
                let char = unsafe { self.char };
                // all u8 is valid char
                unsafe { char::from_u32_unchecked(char as u32).to_pyobject(vm) }
            }

            Type::WChar(_) => {
                let char = unsafe { self.wchar };
                // not all u16 is valid char
                char::from_u32(char as u32).to_pyobject(vm)
            }

            // there's no reasonable way to let jitpoline write sret to Ret
            // so rather than handle here, we just pass in memory it can write to
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct RetMemory {
    ptr: RawSendable<Ret>,
    layout: Layout,
}

impl RetMemory {
    pub fn new() -> Self {
        // Safe cause Ret's size is positive, and size and align are official functions
        let layout = unsafe {
            Layout::from_size_align_unchecked(
                std::mem::size_of::<Ret>(),
                std::mem::align_of::<Ret>(),
            )
        };

        let memory = unsafe { alloc::alloc(layout) };

        Self {
            ptr: RawSendable::new(memory.cast()),
            layout,
        }
    }

    pub fn mem(&self) -> *mut Ret {
        self.ptr.as_ptr()
    }
}

impl Drop for RetMemory {
    fn drop(&mut self) {
        unsafe {
            alloc::dealloc(self.ptr.as_ptr().cast(), self.layout);
        }
    }
}
