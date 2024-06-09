use core::slice;
use std::fmt::{self, Display};

use capstone::prelude::*;
use capstone::Insn;
use keystone_engine::{Arch, Keystone, Mode};

use crate::{Address, AddressUtils as _};

#[derive(Debug, thiserror::Error)]
pub enum AsmError {
    #[error("bad address")]
    BadAddress,
    #[error("failed to assemble asm")]
    BadAsm,
    #[error("failed to disassemble")]
    BadDis,
    #[error("there were no instructions to disassemble")]
    NoInstructions,
    #[error(transparent)]
    Keystone(#[from] keystone_engine::KeystoneError),
    #[error(transparent)]
    Capstone(#[from] capstone::Error),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Inst {
    pub address: u64,
    pub size: usize,
    pub bytes: Vec<u8>,
    pub mnemonic: Option<String>,
    pub op_str: Option<String>,
}

unsafe impl Send for Inst {}

impl<'a> From<&'a Insn<'a>> for Inst {
    fn from(value: &Insn) -> Self {
        Self {
            address: value.address(),
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
                "{mnemonic} {op_str} @ {:#x} -> {:x?}",
                self.address, self.bytes
            )
        } else {
            write!(f, "?? ?? @ {:#x} -> {:x?}", self.address, self.bytes)
        }
    }
}

pub trait InstructionBytes {
    fn to_bytes(&self) -> Vec<u8>;
}

impl InstructionBytes for Vec<Inst> {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        for inst in self.iter() {
            buffer.extend_from_slice(&inst.bytes);
        }

        buffer
    }
}

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

pub fn assemble_ex(code: &str, runtime_addr: Address) -> Result<Vec<Inst>, AsmError> {
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

pub unsafe fn disassemble(addr: Address) -> Result<Inst, AsmError> {
    let cs = Capstone::new()
        .x86()
        .mode(arch::x86::ArchMode::Mode64)
        .syntax(arch::x86::ArchSyntax::Intel)
        .build()?;

    let code = unsafe { slice::from_raw_parts(addr as _, 16) };

    let insts = cs.disasm_count(code, 0, 1)?;

    let Some(inst) = insts.as_ref().iter().next() else {
        return Err(AsmError::NoInstructions);
    };

    Ok(inst.into())
}

pub unsafe fn disassemble_ex(
    addr: Address,
    size: usize,
    runtime_addr: Address,
) -> Result<Vec<Inst>, AsmError> {
    if addr.is_null() {
        return Err(AsmError::BadAddress);
    }

    // provenance valid cause caller asserts it was previously exposed
    let addr = sptr::from_exposed_addr::<u8>(addr);
    let code = unsafe { slice::from_raw_parts(addr, size) };

    disassemble_bytes_ex(code, runtime_addr)
}

pub unsafe fn disassemble_ex_count(
    addr: Address,
    size: usize,
    runtime_addr: Address,
    instruction_count: usize,
) -> Result<Vec<Inst>, AsmError> {
    if addr.is_null() {
        return Err(AsmError::BadAddress);
    }

    // provenance valid cause caller asserts it was previously exposed
    let addr = sptr::from_exposed_addr::<u8>(addr);
    let code = unsafe { slice::from_raw_parts(addr, size) };

    disassemble_bytes_ex_count(code, runtime_addr, instruction_count)
}

pub fn disassemble_bytes(code: &[u8]) -> Result<Vec<Inst>, AsmError> {
    disassemble_bytes_ex(code, 0)
}

pub fn disassemble_bytes_count(
    code: &[u8],
    instruction_count: usize,
) -> Result<Vec<Inst>, AsmError> {
    disassemble_bytes_ex_count(code, 0, instruction_count)
}

pub fn disassemble_bytes_ex(code: &[u8], runtime_addr: Address) -> Result<Vec<Inst>, AsmError> {
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

pub fn disassemble_bytes_ex_count(
    code: &[u8],
    runtime_addr: Address,
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

pub unsafe fn code_len(mut addr: Address, min_len: usize) -> Result<usize, AsmError> {
    if addr.is_null() {
        return Err(AsmError::BadAddress);
    }

    let mut len = 0;
    while len < min_len {
        let Ok(inst) = (unsafe { disassemble(addr) }) else {
            return Ok(0);
        };

        len += inst.size;
        addr += inst.size;
    }

    Ok(len)
}

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
