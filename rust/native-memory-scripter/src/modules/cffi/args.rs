use std::{
    alloc::{self, Layout, LayoutError},
    ffi::CStr,
    sync::Mutex,
};

use rustpython_vm::{
    builtins::PyBytes,
    convert::ToPyObject,
    prelude::{PyObjectRef, PyResult, VirtualMachine},
};
use tracing::{trace, warn};

use super::types::Type;
use crate::{modules::cffi::cffi::WStr, utils::RawSendable};

// Get the layout and offsets for arg list
fn make_layout(args: &[Type]) -> Result<Option<(Layout, Vec<usize>)>, LayoutError> {
    let args = args.iter().filter(|i| !i.is_void());

    if args.clone().count() == 0 {
        return Ok(None);
    }

    let mut offsets = Vec::new();
    let mut layout = unsafe { Layout::from_size_align_unchecked(0, 1) };

    for &field in args {
        let size = field.layout_size();

        assert!(size > 0, "size returned 0. this should not happen");

        let align = {
            let mut align = size;
            // struct types must be 16 byte aligned according to abi
            // https://learn.microsoft.com/en-us/cpp/build/x64-calling-convention?view=msvc-170#parameter-passing
            if field.is_struct_indirect() {
                align = align.max(16);
            }

            align.checked_next_power_of_two().expect("align overflowed")
        };

        let mut new_layout = Layout::from_size_align(size, align)?;
        // important!! pad this layout to the next alignment, otherwise next entries may overwrite previous data!
        new_layout = new_layout.pad_to_align();

        let (new_layout, offset) = layout.extend(new_layout)?;
        layout = new_layout;

        trace!(?field, size, align, offset, "defining layout arg");

        offsets.push(offset);
    }

    let layout = layout.pad_to_align();

    if layout.size() > 0 {
        Ok(Some((layout, offsets)))
    } else {
        Ok(None)
    }
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
        let (layout, offsets) = make_layout(args).expect("layout failed")?;

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

        trace!(?ty, offset, "next arg");

        let ptr = unsafe { self.ptr.add(offset) };

        let arg = match ty {
            Type::F32(_) => {
                let arg = unsafe { *ptr.cast::<f32>() };
                Arg::F32(arg)
            }

            Type::F64(_) => {
                let arg = unsafe { *ptr.cast::<f64>() };
                Arg::F64(arg)
            }

            Type::U8(_) => {
                let arg = unsafe { *ptr.cast::<u8>() };
                Arg::U8(arg)
            }

            Type::U16(_) => {
                let arg = unsafe { *ptr.cast::<u16>() };
                Arg::U16(arg)
            }

            Type::U32(_) => {
                let arg = unsafe { *ptr.cast::<u32>() };
                Arg::U32(arg)
            }

            Type::U64(_) => {
                let arg = unsafe { *ptr.cast::<u64>() };
                Arg::U64(arg)
            }

            Type::U128(_) => {
                let arg = unsafe { *ptr.cast::<u128>() };
                Arg::U128(arg)
            }

            Type::I8(_) => {
                let arg = unsafe { *ptr.cast::<i8>() };
                Arg::I8(arg)
            }

            Type::I16(_) => {
                let arg = unsafe { *ptr.cast::<i16>() };
                Arg::I16(arg)
            }

            Type::I32(_) => {
                let arg = unsafe { *ptr.cast::<i32>() };
                Arg::I32(arg)
            }

            Type::I64(_) => {
                let arg = unsafe { *ptr.cast::<i64>() };
                Arg::I64(arg)
            }

            Type::I128(_) => {
                let arg = unsafe { *ptr.cast::<i128>() };
                Arg::I128(arg)
            }

            Type::Ptr(_) => {
                let arg = unsafe { *ptr.cast::<*const ()>() };
                Arg::Ptr(arg)
            }

            Type::Bool(_) => {
                let arg = unsafe { *ptr.cast::<bool>() };
                Arg::Bool(arg)
            }

            Type::CStr(_) => {
                let ptr = unsafe { *ptr.cast::<*const i8>() };
                Arg::CStr(ptr)
            }

            Type::WStr(_) => {
                let ptr = unsafe { *ptr.cast::<*const u16>() };
                Arg::WStr(ptr)
            }

            Type::Char(_) => {
                let arg = unsafe { *ptr.cast::<u8>() };
                Arg::Char(arg as char)
            }

            Type::WChar(_) => {
                let arg = unsafe { *ptr.cast::<u16>() };
                Arg::WChar(unsafe { char::from_u32_unchecked(arg as u32) })
            }

            Type::Struct(size) => {
                // https://github.com/rust-lang/rust/blob/c1f62a7c35349438ea9728abbe1bcf1cebd426b7/compiler/rustc_target/src/abi/call/x86_win64.rs#L10
                let arg = match size {
                    // read as sized array instead of int in order to keep consistent endianness
                    1 => StructType::I8(unsafe { *ptr.cast::<[u8; 1]>() }),
                    2 => StructType::I16(unsafe { *ptr.cast::<[u8; 2]>() }),
                    4 => StructType::I32(unsafe { *ptr.cast::<[u8; 4]>() }),
                    8 => StructType::I64(unsafe { *ptr.cast::<[u8; 8]>() }),
                    _ => StructType::Ptr(unsafe { *ptr.cast::<*const u8>() }),
                };

                Arg::Struct(*size, arg)
            }

            _ => unreachable!(),
        };

        Some(arg)
    }
}

// https://github.com/rust-lang/rust/blob/c1f62a7c35349438ea9728abbe1bcf1cebd426b7/compiler/rustc_target/src/abi/call/x86_win64.rs#L10
#[derive(Debug, Copy, Clone)]
pub enum StructType {
    I8([u8; 1]),
    I16([u8; 2]),
    I32([u8; 4]),
    I64([u8; 8]),
    Ptr(*const u8),
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

    // StructArg Ptr
    #[allow(clippy::enum_variant_names)]
    Struct(u32, StructType),

    // Bool
    Bool(bool),

    // Strings
    // c str (null terminated) - r64
    CStr(*const i8),
    // utf16 str - r64 (length unknown)
    WStr(*const u16),

    // Characters
    // u8
    Char(char),
    // u16
    WChar(char),
}

impl Arg {
    pub fn to_pyobject(self, vm: &VirtualMachine) -> PyObjectRef {
        match self {
            Arg::F32(f) => f.to_pyobject(vm),
            Arg::F64(f) => f.to_pyobject(vm),
            Arg::U8(u) => u.to_pyobject(vm),
            Arg::U16(u) => u.to_pyobject(vm),
            Arg::U32(u) => u.to_pyobject(vm),
            Arg::U64(u) => u.to_pyobject(vm),
            Arg::U128(u) => u.to_pyobject(vm),
            Arg::I8(i) => i.to_pyobject(vm),
            Arg::I16(i) => i.to_pyobject(vm),
            Arg::I32(i) => i.to_pyobject(vm),
            Arg::I64(i) => i.to_pyobject(vm),
            Arg::I128(i) => i.to_pyobject(vm),
            Arg::Bool(b) => b.to_pyobject(vm),
            Arg::Char(c) => c.to_pyobject(vm),
            Arg::WChar(w) => w.to_pyobject(vm),

            Arg::Ptr(ptr) => (ptr as usize).to_pyobject(vm),
            // no idea what the len is
            Arg::WStr(ptr) => (ptr as usize).to_pyobject(vm),

            Arg::Struct(size, data) => {
                let data = match data {
                    StructType::I8(i) => i.to_vec(),
                    StructType::I16(i) => i.to_vec(),
                    StructType::I32(i) => i.to_vec(),
                    StructType::I64(i) => i.to_vec(),
                    StructType::Ptr(p) => {
                        // SAFETY: Signature creator asserts arg + size is correct
                        unsafe { mem::memory::read_bytes(p, size as usize) }
                    }
                };

                data.to_pyobject(vm)
            }

            Arg::CStr(ptr) => {
                // SAFETY: Signature creator asserts arg is correct
                if ptr.is_null() {
                    None::<()>.to_pyobject(vm)
                } else {
                    let _cstr = unsafe { CStr::from_ptr(ptr) };
                    let _str = _cstr.to_string_lossy();
                    _str.to_pyobject(vm)
                }
            }
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
    is_alloc: bool,
}

impl ArgMemory {
    pub fn new(args: &[Type]) -> Self {
        let Some((layout, offsets)) = make_layout(args).expect("layout failed") else {
            return Self {
                ptr: RawSendable::dangling(),
                layout: unsafe { Layout::from_size_align_unchecked(0, 1) },
                offsets: Vec::new(),
                args: args.to_vec(),
                lock: Mutex::new(()),
                is_alloc: false,
            };
        };

        let memory = unsafe { alloc::alloc(layout) };

        Self {
            ptr: RawSendable::new(memory),
            layout,
            offsets,
            args: args.to_vec(),
            lock: Mutex::new(()),
            is_alloc: true,
        }
    }

    pub fn offsets(&self) -> &[usize] {
        &self.offsets
    }

    unsafe fn write<D>(&self, data: D, offset: usize) {
        unsafe { self.ptr.as_ptr().add(offset).cast::<D>().write(data) }
    }

    unsafe fn write_bytes(&self, data: &[u8], offset: usize) {
        let ptr = unsafe { self.ptr.as_ptr().add(offset) };
        unsafe {
            mem::memory::write_bytes(data, ptr);
        }
    }

    /// fill the block of memory with python args
    /// will lock since it's writing to mutable memory
    pub fn fill(&self, args: &[PyObjectRef], vm: &VirtualMachine) -> PyResult<()> {
        if args.len() != self.args.len() {
            return Err(vm.new_runtime_error(format!(
                "fn defined with {} args, but caller only provided {}",
                self.args.len(),
                args.len()
            )));
        }

        if self.args.is_empty() {
            return Ok(());
        }

        let _lock = self.lock.lock().unwrap();

        for (arg, (offset, ty)) in args
            .iter()
            .zip(self.offsets.iter().copied().zip(self.args.iter().copied()))
        {
            trace!(?ty, offset, arg = %arg.str(vm)?, "filling arg");

            match ty {
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

                Type::Ptr(_) => {
                    let p = arg.try_to_value::<usize>(vm)?;

                    unsafe {
                        self.write(p, offset);
                    }
                }

                Type::WStr(_) => {
                    let wstr = arg
                        .downcast_ref::<WStr>()
                        .ok_or_else(|| vm.new_type_error("failed to convert to WStr".to_owned()))?;

                    let ptr = wstr.as_ptr();

                    unsafe {
                        self.write(ptr, offset);
                    }
                }

                Type::CStr(_) => {
                    let string = arg.str(vm)?;
                    let pystr = string.as_str();

                    if !pystr.ends_with('\0') {
                        // return empty string with null byte.. At least it's safer than UB
                        warn!("cstr argument does not end with null byte; this will cause UB, so empty cstr returned instead");

                        let null = "\0";
                        unsafe {
                            self.write(null.as_ptr(), offset);
                        }
                    } else {
                        let ptr = pystr.as_ptr();

                        unsafe {
                            self.write(ptr, offset);
                        }
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

                    let mut chars = s.chars();

                    let Some(char) = chars.next() else {
                        return Err(vm.new_runtime_error("string has no char".to_string()));
                    };

                    if chars.next().is_some() {
                        return Err(vm.new_runtime_error("string is not a char".to_string()));
                    }

                    let char = if char.len_utf8() == 1 {
                        char as u8
                    } else {
                        return Err(
                            vm.new_overflow_error("string cannot be encoded into Char".to_string())
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

                    let mut chars = s.chars();

                    let Some(char) = chars.next() else {
                        return Err(vm.new_runtime_error("string has no char".to_string()));
                    };

                    if chars.next().is_some() {
                        return Err(vm.new_runtime_error("string is not a char".to_string()));
                    }

                    let wchar = if char.len_utf16() == 1 {
                        // encode_utf16 just does a cast for a single code unit,
                        // and we've already checked it fits only into 1
                        char as u16
                    } else {
                        return Err(vm.new_overflow_error(
                            "string cannot be encoded into WChar".to_string(),
                        ));
                    };

                    unsafe {
                        self.write(wchar, offset);
                    }
                }

                Type::Struct(size) => {
                    let bytes =
                        arg.clone().bytes(vm)?.downcast::<PyBytes>().map_err(|_| {
                            vm.new_type_error("failed to convert to bytes".to_owned())
                        })?;

                    if bytes.len() != size as usize {
                        return Err(vm.new_runtime_error(format!(
                            "Struct arg expected len {size}, instead got {}",
                            bytes.len()
                        )));
                    }

                    unsafe {
                        self.write_bytes(&bytes, offset);
                    }
                }

                _ => unreachable!(),
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
        if self.is_alloc {
            unsafe {
                alloc::dealloc(self.ptr.as_ptr(), self.layout);
            }
        }
    }
}
