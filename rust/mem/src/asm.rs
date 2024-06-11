//! This module allows one to assemble and disassemble asm

use core::slice;
use std::fmt::{self, Display};

use capstone::prelude::*;
use capstone::Insn;
use keystone_engine::{Arch, Keystone, Mode};

/// An error for the [asm](crate::asm) module
#[derive(Copy, Clone, Debug, thiserror::Error)]
pub enum AsmError {
    /// an invalid address
    #[error("bad address")]
    BadAddress,
    /// the asm could not be compiled
    #[error("failed to assemble asm")]
    BadAsm,
    /// something wrong happened with disassembly
    #[error("failed to disassemble")]
    BadDis,
    /// no instructions were able to be disassembled
    #[error("there were no instructions to disassemble")]
    NoInstructions,
    /// a keystone error
    #[error(transparent)]
    Keystone(#[from] keystone_engine::KeystoneError),
    /// a capstone error
    #[error(transparent)]
    Capstone(#[from] capstone::Error),
}

/// A single compiled instruction
#[derive(Debug, Clone, PartialEq)]
pub struct Inst {
    /// the base address of the instruction (if you used `runtime_addr`)
    pub address: *const u8,
    /// the byte size of the instruction
    pub size: usize,
    /// the instruction bytes
    pub bytes: Vec<u8>,
    /// the mnemonic
    pub mnemonic: Option<String>,
    /// the op str
    pub op_str: Option<String>,
}

unsafe impl Send for Inst {}

impl<'a> From<&'a Insn<'a>> for Inst {
    fn from(value: &Insn) -> Self {
        Self {
            address: value.address() as _,
            size: value.len(),
            bytes: value.bytes().to_vec(),
            mnemonic: value.mnemonic().map(ToOwned::to_owned),
            op_str: value.op_str().map(ToOwned::to_owned),
        }
    }
}

impl Display for Inst {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let (Some(mnemonic), Some(op_str)) = (self.mnemonic.as_deref(), self.op_str.as_deref()) {
            write!(
                f,
                "{mnemonic} {op_str} @ {:?} -> {:x?}",
                self.address, self.bytes
            )
        } else {
            write!(f, "?? ?? @ {:?} -> {:x?}", self.address, self.bytes)
        }
    }
}

/// Allow a vector of instructions to be converted to bytes
pub trait InstructionBytes {
    /// Convert all instructions to bytes
    fn to_bytes(&self) -> Vec<u8>;
}

impl InstructionBytes for Vec<Inst> {
    /// Convert all instructions to bytes
    fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        for inst in self.iter() {
            buffer.extend_from_slice(&inst.bytes);
        }

        buffer
    }
}

/// Assemble code string to instructions
///
/// ```rust,ignore
/// assemble("jmp [rip]");
/// ```
pub fn assemble(code: &str) -> Result<Inst, AsmError> {
    if code.is_empty() {
        return Err(AsmError::BadAsm);
    }

    let ks = Keystone::new(Arch::X86, Mode::MODE_64)?;

    let output = ks.asm(code.into(), 0)?;

    if output.bytes.is_empty() {
        return Err(AsmError::BadAsm);
    }

    let cs = Capstone::new()
        .x86()
        .mode(arch::x86::ArchMode::Mode64)
        .syntax(arch::x86::ArchSyntax::Intel)
        .build()?;

    let insts = cs.disasm_count(&output.bytes, 0, 1)?;

    let Some(inst) = insts.as_ref().iter().next() else {
        return Err(AsmError::NoInstructions);
    };

    Ok(inst.into())
}

/// Assemble code string to instructions, with runtime_addr
///
/// ```rust,ignore
/// assemble("jmp [rip]", 0x112233445566);
/// ```
pub fn assemble_ex(code: &str, runtime_addr: usize) -> Result<Vec<Inst>, AsmError> {
    if code.is_empty() {
        return Err(AsmError::BadAsm);
    }

    let ks = Keystone::new(Arch::X86, Mode::MODE_64)?;

    let output = ks.asm(code.into(), runtime_addr as u64)?;

    if output.bytes.is_empty() {
        return Err(AsmError::BadAsm);
    }

    let dis = disassemble_bytes_ex(&output.bytes, runtime_addr)?;

    Ok(dis)
}

/// Disassemble a single instruction at the target address
///
/// # Safety
/// - Address must be valid ptr
/// - Address must be valid for max 16 bytes read
pub unsafe fn disassemble(addr: *const u8) -> Result<Inst, AsmError> {
    let cs = Capstone::new()
        .x86()
        .mode(arch::x86::ArchMode::Mode64)
        .syntax(arch::x86::ArchSyntax::Intel)
        .build()?;

    let code = unsafe { slice::from_raw_parts(addr, 16) };

    let insts = cs.disasm_count(code, 0, 1)?;

    let Some(inst) = insts.as_ref().iter().next() else {
        return Err(AsmError::NoInstructions);
    };

    Ok(inst.into())
}

/// Disassemble all instructions from target address up to `size`, with
/// optional runtime_addr to make detecting actual address easy
///
/// # Safety
/// - Address must be valid ptr
/// - Address must be valid for `size` bytes read
pub unsafe fn disassemble_ex(
    addr: *const u8,
    size: usize,
    runtime_addr: usize,
) -> Result<Vec<Inst>, AsmError> {
    if addr.is_null() {
        return Err(AsmError::BadAddress);
    }

    let code = unsafe { slice::from_raw_parts(addr, size) };

    disassemble_bytes_ex(code, runtime_addr)
}

/// Disassemble up to `instruction_count` instructions from target address up to `size`, with
/// optional runtime_addr to make detecting actual address easy
///
/// # Safety
/// - Address must be valid ptr
/// - Address must be valid for `size` bytes read
pub unsafe fn disassemble_ex_count(
    addr: *const u8,
    size: usize,
    runtime_addr: usize,
    instruction_count: usize,
) -> Result<Vec<Inst>, AsmError> {
    if addr.is_null() {
        return Err(AsmError::BadAddress);
    }

    let code = unsafe { slice::from_raw_parts(addr, size) };

    disassemble_bytes_ex_count(code, runtime_addr, instruction_count)
}

/// Disassemble all bytes in `code` into instructions, with `runtime_addr` of 0
pub fn disassemble_bytes(code: &[u8]) -> Result<Vec<Inst>, AsmError> {
    disassemble_bytes_ex(code, 0)
}

/// Disassemble up to `instruction_count` instructions from bytes
pub fn disassemble_bytes_count(
    code: &[u8],
    instruction_count: usize,
) -> Result<Vec<Inst>, AsmError> {
    disassemble_bytes_ex_count(code, 0, instruction_count)
}

/// Disassemble all instructions from bytes, with`runtime_addr`
pub fn disassemble_bytes_ex(code: &[u8], runtime_addr: usize) -> Result<Vec<Inst>, AsmError> {
    let cs = Capstone::new()
        .x86()
        .mode(arch::x86::ArchMode::Mode64)
        .syntax(arch::x86::ArchSyntax::Intel)
        .build()?;

    let insts = cs.disasm_all(code, runtime_addr as u64)?;

    let mut buffer = Vec::new();
    for inst in insts.as_ref() {
        let inst: Inst = inst.into();
        buffer.push(inst);
    }

    Ok(buffer)
}

/// Disassemble up to `instruction_count` instructions from bytes, with`runtime_addr`
pub fn disassemble_bytes_ex_count(
    code: &[u8],
    runtime_addr: usize,
    instruction_count: usize,
) -> Result<Vec<Inst>, AsmError> {
    let cs = Capstone::new()
        .x86()
        .mode(arch::x86::ArchMode::Mode64)
        .syntax(arch::x86::ArchSyntax::Intel)
        .build()?;

    let insts = cs.disasm_count(code, runtime_addr as u64, instruction_count)?;

    let mut buffer = Vec::new();
    for inst in insts.as_ref() {
        let inst: Inst = inst.into();
        buffer.push(inst);
    }

    Ok(buffer)
}

/// Starting at address, get the closest valid length of bytes to `min_len` without overwriting any asm instructions
///
/// # Safety
/// - address must be valid address for reads for min_len up to the returned usize (16 bytes max)
pub unsafe fn code_len(mut addr: *const u8, min_len: usize) -> Result<usize, AsmError> {
    if addr.is_null() {
        return Err(AsmError::BadAddress);
    }

    let mut len = 0;
    while len < min_len {
        let Ok(inst) = (unsafe { disassemble(addr) }) else {
            return Ok(0);
        };

        len += inst.size;
        addr = unsafe { addr.add(inst.size) };
    }

    Ok(len)
}

/// Get the closest valid length of bytes in byte slice to `min_len` without overwriting any asm instructions
pub fn code_bytes_len(bytes: &[u8], min_len: usize) -> Result<usize, AsmError> {
    let insts = disassemble_bytes(bytes)?;

    let mut len = 0;
    for inst in insts {
        len += inst.size;

        if len >= min_len {
            break;
        }
    }

    Ok(len)
}
