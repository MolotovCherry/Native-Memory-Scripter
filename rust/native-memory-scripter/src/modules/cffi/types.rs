use std::fmt;

use cranelift::prelude::types::{self, Type as CType};

#[derive(Copy, Clone)]
pub enum Type {
    Void,

    // Floats
    F32(CType),
    F64(CType),

    // Unsigned
    U8(CType),
    U16(CType),
    U32(CType),
    U64(CType),
    U128(CType),

    // Integers
    I8(CType),
    I16(CType),
    I32(CType),
    I64(CType),
    I128(CType),

    // Pointer
    Ptr(CType),

    // Bool
    Bool(CType),

    // Strings
    // c str (null terminated) - i64
    CStr(CType),
    // utf16 str (null terminated) - i64
    WStr(CType),

    // Characters
    // i8
    Char(CType),
    // i16
    WChar(CType),

    // by value struct
    Struct(u32),
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::Void => f.write_str("Void"),
            Type::F32(_) => f.write_str("F32"),
            Type::F64(_) => f.write_str("F64"),
            Type::U8(_) => f.write_str("U8"),
            Type::U16(_) => f.write_str("U16"),
            Type::U32(_) => f.write_str("U32"),
            Type::U64(_) => f.write_str("U64"),
            Type::U128(_) => f.write_str("U128"),
            Type::I8(_) => f.write_str("I8"),
            Type::I16(_) => f.write_str("I16"),
            Type::I32(_) => f.write_str("I32"),
            Type::I64(_) => f.write_str("I64"),
            Type::I128(_) => f.write_str("I128"),
            Type::Ptr(_) => f.write_str("Ptr"),
            Type::Bool(_) => f.write_str("Bool"),
            Type::CStr(_) => f.write_str("CStr"),
            Type::WStr(_) => f.write_str("WStr"),
            Type::Char(_) => f.write_str("Char"),
            Type::WChar(_) => f.write_str("WChar"),
            Type::Struct(f0) => f.debug_tuple("Struct").field(f0).finish(),
        }
    }
}

impl Type {
    #[inline]
    pub fn is_void(&self) -> bool {
        matches!(self, Self::Void)
    }

    // is this a struct?
    #[inline]
    pub fn is_struct(&self) -> bool {
        matches!(self, Self::Struct(_))
    }

    /// is this struct a ptr?
    /// note: fastcall
    #[inline]
    pub fn is_struct_indirect(self) -> bool {
        match self {
            // size > 8 or not power of 2 is always a ptr
            Self::Struct(size) => size > 8 || !size.is_power_of_two(),
            _ => false,
        }
    }

    pub fn layout_size(self) -> usize {
        match self {
            Type::Void => 0,

            // how man bytes we are going to store, which is simply `size`
            Type::Struct(size) => size as usize,

            t => {
                let ty: CType = t.into();
                ty.bytes() as usize
            }
        }
    }
}

impl From<Type> for CType {
    fn from(val: Type) -> Self {
        match val {
            Type::F32(t)
            | Type::F64(t)
            | Type::U8(t)
            | Type::U16(t)
            | Type::U32(t)
            | Type::U64(t)
            | Type::U128(t)
            | Type::I8(t)
            | Type::I16(t)
            | Type::I32(t)
            | Type::I64(t)
            | Type::I128(t)
            | Type::Ptr(t)
            | Type::Bool(t)
            | Type::CStr(t)
            | Type::WStr(t)
            | Type::Char(t)
            | Type::WChar(t) => t,

            Type::Struct(size) => match size {
                // https://github.com/rust-lang/rust/blob/c1f62a7c35349438ea9728abbe1bcf1cebd426b7/compiler/rustc_target/src/abi/call/x86_win64.rs#L10
                1 => types::I8,
                2 => types::I16,
                4 => types::I32,
                8 => types::I64,
                // it's a ptr!
                _ => types::I64,
            },

            // this means we didn't properly handle code somewhere
            _ => unreachable!("bug: invalid type"),
        }
    }
}
