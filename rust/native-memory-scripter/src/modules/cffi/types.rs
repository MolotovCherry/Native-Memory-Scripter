use cranelift::prelude::types::{self, Type as CType};

#[derive(Debug, Copy, Clone)]
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
    // c str (null terminated) - r64
    CStr(CType),
    // utf16 str (null terminated) - r64
    WStr(CType),

    // Characters
    // i8
    Char(CType),
    // i16
    WChar(CType),

    // by value struct - only valid as arg
    StructArg(u32),

    // struct return - only valid as return type
    StructReturn(u32),
}

impl Type {
    #[inline]
    pub fn is_void(&self) -> bool {
        matches!(self, Self::Void)
    }

    #[inline]
    pub fn is_sret(&self) -> bool {
        match self {
            Self::StructReturn(size) => !matches!(size, 1 | 2 | 4 | 8),
            _ => false,
        }
    }

    #[inline]
    pub fn is_sarg(&self) -> bool {
        matches!(self, Self::StructArg(_))
    }

    // not valid for structreturn
    pub fn size(&self) -> usize {
        match self {
            Type::Void => 0,

            // size of a ptr
            Type::StructArg(_) | Type::StructReturn(_) => 8,

            &t => {
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

            Type::StructArg(size) | Type::StructReturn(size) => match size {
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
