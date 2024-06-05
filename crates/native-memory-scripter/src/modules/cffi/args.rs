use std::{
    alloc::{self, Layout},
    ptr::NonNull,
    sync::Mutex,
};

use rustpython_vm::prelude::{PyObjectRef, PyResult, VirtualMachine};
use tracing::error;

use super::types::Type;
use crate::utils::RawSendable;

// Get the layout and offsets for arg list
fn get_layout(args: &[Type]) -> Option<(Layout, Vec<usize>)> {
    if args.is_empty() {
        return None;
    }

    let mut offsets = Vec::new();
    let mut layout = unsafe { Layout::from_size_align_unchecked(0, 1) };
    for &field in args {
        let size = field.size();

        let field = unsafe { Layout::from_size_align_unchecked(size, size) };

        let (new_layout, offset) = layout.extend(field).ok()?;
        layout = new_layout;

        offsets.push(offset);
    }

    let layout = layout.pad_to_align();

    (layout.size() > 0).then_some((layout, offsets))
}

#[derive(Debug, Clone)]
pub struct ArgLayout {
    args: Vec<Type>,
    offsets: Vec<usize>,
    size: usize,
    align: usize,
}

impl ArgLayout {
    pub fn new(args: &[Type]) -> Option<Self> {
        let (layout, offsets) = get_layout(args)?;

        Some(Self {
            size: layout.size() as _,
            args: args.to_vec(),
            offsets,
            align: layout.align() as _,
        })
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn align(&self) -> usize {
        self.align
    }

    pub fn offsets(&self) -> &[usize] {
        &self.offsets
    }

    /// SAFETY:
    /// ptr must be valid for type+offset reads for anything in self.offsets and self.args
    pub unsafe fn iter(&self, ptr: *const ()) -> ArgLayoutIterator {
        ArgLayoutIterator {
            ptr: ptr.cast(),
            layout: self,
            pos: 0,
        }
    }
}

pub struct ArgLayoutIterator<'a> {
    ptr: *const u8,
    layout: &'a ArgLayout,
    pos: usize,
}

impl<'a> Iterator for ArgLayoutIterator<'a> {
    type Item = Arg;

    fn next(&mut self) -> Option<Self::Item> {
        let pos = self.pos;
        self.pos += 1;

        let ty = self.layout.args.get(pos)?;
        let offset = self.layout.offsets[pos];

        let arg = match ty {
            Type::Void => {
                error!("bug! args cannot be void!");
                return None;
            }

            Type::F32(_) => {
                let arg = unsafe { *self.ptr.add(offset).cast::<f32>() };
                Arg::F32(arg)
            }

            Type::F64(_) => {
                let arg = unsafe { *self.ptr.add(offset).cast::<f64>() };
                Arg::F64(arg)
            }

            Type::U8(_) => {
                let arg = unsafe { *self.ptr.add(offset).cast::<u8>() };
                Arg::U8(arg)
            }

            Type::U16(_) => {
                let arg = unsafe { *self.ptr.add(offset).cast::<u16>() };
                Arg::U16(arg)
            }

            Type::U32(_) => {
                let arg = unsafe { *self.ptr.add(offset).cast::<u32>() };
                Arg::U32(arg)
            }

            Type::U64(_) => {
                let arg = unsafe { *self.ptr.add(offset).cast::<u64>() };
                Arg::U64(arg)
            }

            Type::U128(_) => {
                let arg = unsafe { *self.ptr.add(offset).cast::<u128>() };
                Arg::U128(arg)
            }

            Type::I8(_) => {
                let arg = unsafe { *self.ptr.add(offset).cast::<i8>() };
                Arg::I8(arg)
            }

            Type::I16(_) => {
                let arg = unsafe { *self.ptr.add(offset).cast::<i16>() };
                Arg::I16(arg)
            }

            Type::I32(_) => {
                let arg = unsafe { *self.ptr.add(offset).cast::<i32>() };
                Arg::I32(arg)
            }

            Type::I64(_) => {
                let arg = unsafe { *self.ptr.add(offset).cast::<i64>() };
                Arg::I64(arg)
            }

            Type::I128(_) => {
                let arg = unsafe { *self.ptr.add(offset).cast::<i128>() };
                Arg::I128(arg)
            }

            Type::Ptr(_) => {
                let arg = unsafe { *self.ptr.add(offset).cast::<*const ()>() };
                Arg::Ptr(arg)
            }

            Type::Bool(_) => {
                let arg = unsafe { *self.ptr.add(offset).cast::<bool>() };
                Arg::Bool(arg)
            }

            Type::CStr(_) => {
                let ptr = unsafe { *self.ptr.add(offset).cast::<*const u8>() };
                Arg::CStr(ptr)
            }

            Type::WStr(_) => {
                let ptr = unsafe { *self.ptr.add(offset).cast::<*const u16>() };
                Arg::WStr(ptr)
            }

            Type::Char(_) => {
                let arg = unsafe { *self.ptr.add(offset).cast::<u8>() };
                Arg::Char(arg as char)
            }

            Type::WChar(_) => {
                let arg = unsafe { *self.ptr.add(offset).cast::<u16>() };
                Arg::WChar(unsafe { char::from_u32_unchecked(arg as u32) })
            }
        };

        Some(arg)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Arg {
    // Floats
    F32(f32),
    F64(f64),

    // Unsigned
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),

    // Integers
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),

    // Pointer
    Ptr(*const ()),

    // Bool
    Bool(bool),

    // Strings
    // c str (null terminated) - r64
    CStr(*const u8),
    // utf16 str - r64 (length unknown)
    WStr(*const u16),

    // Characters
    // u8
    Char(char),
    // u16
    WChar(char),
}

impl Arg {
    pub fn as_f32(&self) -> f32 {
        match self {
            Self::F32(f) => *f,
            _ => unreachable!(),
        }
    }

    pub fn as_f64(&self) -> f64 {
        match self {
            Self::F64(f) => *f,
            _ => unreachable!(),
        }
    }

    pub fn as_u8(&self) -> u8 {
        match self {
            Self::U8(u) => *u,
            _ => unreachable!(),
        }
    }

    pub fn as_u16(&self) -> u16 {
        match self {
            Self::U16(u) => *u,
            _ => unreachable!(),
        }
    }

    pub fn as_u32(&self) -> u32 {
        match self {
            Self::U32(u) => *u,
            _ => unreachable!(),
        }
    }

    pub fn as_u64(&self) -> u64 {
        match self {
            Self::U64(u) => *u,
            _ => unreachable!(),
        }
    }

    pub fn as_i8(&self) -> i8 {
        match self {
            Self::I8(i) => *i,
            _ => unreachable!(),
        }
    }

    pub fn as_i16(&self) -> i16 {
        match self {
            Self::I16(i) => *i,
            _ => unreachable!(),
        }
    }

    pub fn as_i32(&self) -> i32 {
        match self {
            Self::I32(i) => *i,
            _ => unreachable!(),
        }
    }

    pub fn as_i64(&self) -> i64 {
        match self {
            Self::I64(i) => *i,
            _ => unreachable!(),
        }
    }

    pub fn as_ptr(&self) -> *const () {
        match self {
            Self::Ptr(p) => *p,
            _ => unreachable!(),
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Self::Bool(b) => *b,
            _ => unreachable!(),
        }
    }

    pub fn as_cstr(&self) -> *const u8 {
        match self {
            Self::CStr(c) => *c,
            _ => unreachable!(),
        }
    }

    pub fn as_wstr(&self) -> *const u16 {
        match self {
            Self::WStr(w) => *w,
            _ => unreachable!(),
        }
    }

    pub fn as_char(&self) -> char {
        match self {
            Self::Char(c) => *c as char,
            _ => unreachable!(),
        }
    }

    pub fn as_wchar(&self) -> char {
        match self {
            Self::WChar(c) => *c,
            _ => unreachable!(),
        }
    }
}

/// Creates a dynamic struct in memory for passing args to jit
#[derive(Debug)]
pub struct ArgMemory {
    ptr: RawSendable<u8>,
    layout: Layout,
    offsets: Vec<usize>,
    args: Vec<Type>,
    lock: Mutex<()>,
}

impl ArgMemory {
    pub fn new(args: &[Type]) -> Option<Self> {
        let (layout, offsets) = get_layout(args)?;

        let memory = unsafe { alloc::alloc(layout) };

        let slf = Self {
            ptr: RawSendable(unsafe { NonNull::new_unchecked(memory) }),
            layout,
            offsets,
            args: args.to_vec(),
            lock: Mutex::new(()),
        };

        Some(slf)
    }

    pub fn offsets(&self) -> &[usize] {
        &self.offsets
    }

    unsafe fn write<D>(&self, data: D, offset: usize) {
        unsafe { self.ptr.as_ptr().add(offset).cast::<D>().write(data) }
    }

    /// fill the block of memory with python args
    /// will lock since it's writing to mutable memory
    pub fn fill(&self, args: &[PyObjectRef], vm: &VirtualMachine) -> PyResult<()> {
        if args.len() != self.args.len() {
            return Err(vm.new_runtime_error("incorrect number of args".to_owned()));
        }

        let _lock = self.lock.lock().unwrap();

        for (arg, (offset, ty)) in args
            .iter()
            .zip(self.offsets.iter().copied().zip(self.args.iter().copied()))
        {
            match ty {
                Type::Void => {
                    return Err(vm.new_runtime_error("void cannot be an argument".to_owned()));
                }

                Type::F32(_) => {
                    // note: truncation happens if f64 was beyond its limits
                    let f = arg.try_float(vm)?.to_f64() as f32;

                    unsafe {
                        self.write(f, offset);
                    }
                }

                Type::F64(_) => {
                    let f = arg.try_float(vm)?.to_f64();

                    unsafe {
                        self.write(f, offset);
                    }
                }

                Type::U8(_) => {
                    let u = arg.try_to_value::<u8>(vm)?;

                    unsafe {
                        self.write(u, offset);
                    }
                }

                Type::U16(_) => {
                    let u = arg.try_to_value::<u16>(vm)?;

                    unsafe {
                        self.write(u, offset);
                    }
                }

                Type::U32(_) => {
                    let u = arg.try_to_value::<u32>(vm)?;

                    unsafe {
                        self.write(u, offset);
                    }
                }

                Type::U64(_) => {
                    let u = arg.try_to_value::<u64>(vm)?;

                    unsafe {
                        self.write(u, offset);
                    }
                }

                Type::U128(_) => {
                    let u = arg.try_to_value::<u128>(vm)?;

                    unsafe {
                        self.write(u, offset);
                    }
                }

                Type::I8(_) => {
                    let i = arg.try_to_value::<i8>(vm)?;

                    unsafe {
                        self.write(i, offset);
                    }
                }

                Type::I16(_) => {
                    let i = arg.try_to_value::<i16>(vm)?;

                    unsafe {
                        self.write(i, offset);
                    }
                }

                Type::I32(_) => {
                    let i = arg.try_to_value::<i32>(vm)?;

                    unsafe {
                        self.write(i, offset);
                    }
                }

                Type::I64(_) => {
                    let i = arg.try_to_value::<i64>(vm)?;

                    unsafe {
                        self.write(i, offset);
                    }
                }

                Type::I128(_) => {
                    let i = arg.try_to_value::<i128>(vm)?;

                    unsafe {
                        self.write(i, offset);
                    }
                }

                Type::Ptr(_) | Type::CStr(_) | Type::WStr(_) => {
                    let p = arg.try_to_value::<usize>(vm)?;

                    unsafe {
                        self.write(p, offset);
                    }
                }

                Type::Bool(_) => {
                    let b = arg.try_to_value::<bool>(vm)?;

                    unsafe {
                        self.write(b as u8, offset);
                    }
                }

                Type::Char(_) => {
                    let s = arg.clone().try_into_value::<String>(vm)?;

                    if s.is_empty() {
                        return Err(vm.new_runtime_error("string cannot be empty".to_string()));
                    }

                    let Some(char) = s.chars().next() else {
                        return Err(vm.new_runtime_error("string has no char".to_string()));
                    };

                    let char = if char.len_utf8() == 1 {
                        char as u8
                    } else {
                        return Err(
                            vm.new_overflow_error("string cannot be encoded into char".to_string())
                        );
                    };

                    unsafe {
                        self.write(char, offset);
                    }
                }

                Type::WChar(_) => {
                    let s = arg.clone().try_into_value::<String>(vm)?;

                    if s.is_empty() {
                        return Err(vm.new_runtime_error("string cannot be empty".to_string()));
                    }

                    let Some(char) = s.chars().next() else {
                        return Err(vm.new_runtime_error("string has no char".to_string()));
                    };

                    let wchar = if char.len_utf16() == 1 {
                        // encode_utf16 just does a cast for a single code unit,
                        // and we've already checked it fits only into 1
                        char as u16
                    } else {
                        return Err(vm.new_overflow_error(
                            "string cannot be encoded into single utf16 char".to_string(),
                        ));
                    };

                    unsafe {
                        self.write(wchar, offset);
                    }
                }
            }
        }

        Ok(())
    }

    /// While it's not unsafe to get the ptr, using it is
    /// You MUST have called fill() before you attempt to read args from the memory
    /// You MUST also NEVER read to this memory while calling fill()
    /// Never write to this memory. Use provided methods
    /// Do not read it after ArgMemory has been dropped
    pub fn mem(&self) -> *const () {
        self.ptr.as_ptr() as _
    }
}

impl Drop for ArgMemory {
    fn drop(&mut self) {
        unsafe {
            alloc::dealloc(self.ptr.as_ptr(), self.layout);
        }
    }
}
