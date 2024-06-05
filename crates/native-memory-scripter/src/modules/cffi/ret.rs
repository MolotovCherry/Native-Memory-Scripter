use rustpython_vm::{
    convert::ToPyObject,
    prelude::{PyObjectRef, VirtualMachine},
};

use super::types::Type;

#[allow(improper_ctypes_definitions)]
#[repr(C)]
pub union Ret {
    pub void: (),

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
    /// SAFETY:
    /// Caller asserts that Type represents the type held in Ret
    pub unsafe fn to_pyobject(&self, ret: Type, vm: &VirtualMachine) -> Option<PyObjectRef> {
        let val = match ret {
            Type::Void => return None,
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
                let ptr = unsafe { self.ptr as *const i8 };
                let data = unsafe { std::ffi::CStr::from_ptr(ptr) };
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
                return char::from_u32(char as u32).map(|c| c.to_pyobject(vm));
            }
        };

        Some(val)
    }
}
