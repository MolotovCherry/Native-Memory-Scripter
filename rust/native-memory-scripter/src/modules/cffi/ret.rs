use std::ffi;

use rustpython_vm::{
    builtins::PyBytes,
    convert::ToPyObject,
    prelude::{PyObjectRef, VirtualMachine},
    PyResult,
};
use tracing::warn;

use super::types::Type;

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
                    let a = val.str(vm)?;

                    Ret {
                        ptr: a.as_str().as_ptr() as usize,
                    }
                }

                // User returns a utf16 str.
                //
                // ```py
                // s = "Hello world!"
                // utf16_str = s.encode('utf-16')
                // ```
                //
                // SAFETY:
                // - If api requires it, it must be null terminated
                // - If api requires it, it must be correct length you specified to api
                // - The lifetime of the python string object must be >= how long it will be used in C
                Type::WStr(_) => {
                    let bytes = val.bytes(vm)?.downcast::<PyBytes>().map_err(|_| {
                        vm.new_type_error("failed to convert to PyBytes".to_owned())
                    })?;

                    Ret {
                        ptr: bytes.as_bytes().as_ptr() as usize,
                    }
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
                            *ret = Ret {
                                i8: bytes.as_ptr().cast::<i8>().read(),
                            }
                        },

                        2 => unsafe {
                            let bytes = bytes.as_ptr().cast::<i16>().read_unaligned();
                            *ret = Ret { i16: bytes }
                        },

                        4 => unsafe {
                            let bytes = bytes.as_ptr().cast::<i32>().read_unaligned();
                            *ret = Ret { i32: bytes }
                        },

                        8 => unsafe {
                            let bytes = bytes.as_ptr().cast::<i64>().read_unaligned();
                            *ret = Ret { i64: bytes }
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

    pub fn write_default_ret(ty: Type, ret: *mut Ret) {
        warn!("python cb failed to execute. default return was triggered to return *something*. this is a bug in your code and is causing unintended results or ub. it should be fixed asap. please ensure you have exception handling code and a reasonable return value for every possible exception that could happen. uncaught exceptions are not allowed in the callback");

        let data = match ty {
            Type::Void => return,

            Type::F32(_) => Ret { f32: 0.0 },
            Type::F64(_) => Ret { f64: 0.0 },
            Type::U8(_) => Ret { u8: 0 },
            Type::U16(_) => Ret { u16: 0 },
            Type::U32(_) => Ret { u32: 0 },
            Type::U64(_) => Ret { u64: 0 },
            Type::U128(_) => Ret { u128: 0 },
            Type::I8(_) => Ret { i8: 0 },
            Type::I16(_) => Ret { i16: 0 },
            Type::I32(_) => Ret { i32: 0 },
            Type::I64(_) => Ret { i64: 0 },
            Type::I128(_) => Ret { i128: 0 },
            // Warning: null ptr return!
            Type::Ptr(_) => Ret { ptr: 0 },
            Type::Bool(_) => Ret { bool: false },
            // Warning: null ptr return!
            Type::CStr(_) => Ret { ptr: 0 },
            // Warning: null ptr return!
            Type::WStr(_) => Ret { ptr: 0 },
            Type::Char(_) => Ret { char: 0 },
            Type::WChar(_) => Ret { wchar: 0 },

            // Warning: zeroed data written to return ptr. Probably UB
            Type::Struct(size) => {
                match size {
                    1 => unsafe { *ret = Ret { i8: 0 } },

                    2 => unsafe { *ret = Ret { i16: 0 } },

                    4 => unsafe { *ret = Ret { i32: 0 } },

                    8 => unsafe { *ret = Ret { i64: 0 } },

                    // it's a ptr!
                    _ => unsafe {
                        mem::memory::set(ret.cast(), 0, size as usize);
                    },
                }

                return;
            }
        };

        // SAFETY: There is none. User must ensure they always return valid values. This may or may not cause UB
        unsafe {
            *ret = data;
        }
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
                let data = unsafe { ffi::CStr::from_ptr(ptr) };
                let string = data.to_string_lossy().to_string();
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
