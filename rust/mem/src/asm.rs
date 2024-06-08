use core::slice;
use std::fmt::{self, Display};

use capstone::prelude::*;
use capstone::Insn;
use keystone_engine::{Arch, Keystone, Mode};

#[derive(Debug, thiserror::Error)]
pub enum AsmError {
    #[error("bad address")]
    BadAddress,
    #[error("failed to assemble asm")]
    BadAsm,
    #[error("failed to disassemble")]
    BadDis,
    #[error(transparent)]
    Keystone(#[from] keystone_engine::KeystoneError),
    #[error(transparent)]
    Capstone(#[from] capstone::Error),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Inst {
    pub id: u32,
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
            id: value.id().0,
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

pub fn assemble(code: &str) -> Result<Vec<Inst>, AsmError> {
    assemble_ex(code, 0, 0)
}

pub fn assemble_ex(
    code: &str,
    runtime_addr: usize,
    instruction_count: usize,
) -> Result<Vec<Inst>, AsmError> {
    if code.is_empty() {
        return Err(AsmError::BadAsm);
    }

    let ks = Keystone::new(Arch::X86, Mode::MODE_64)?;

    let output = ks.asm(code.into(), runtime_addr as u64)?;

    if output.bytes.is_empty() {
        return Err(AsmError::BadAsm);
    }

    let dis = disassemble_bytes_ex(&output.bytes, runtime_addr, instruction_count)?;

    Ok(dis)
}

pub unsafe fn disassemble(addr: *const u8) -> Result<Inst, AsmError> {
    let mut dis = unsafe { disassemble_ex(addr, 16, 0, 1)? };

    if !dis.is_empty() {
        // remove panics, so we need to do the check
        Ok(dis.remove(0))
    } else {
        Err(AsmError::BadDis)
    }
}

pub unsafe fn disassemble_ex(
    addr: *const u8,
    size: usize,
    runtime_addr: usize,
    instruction_count: usize,
) -> Result<Vec<Inst>, AsmError> {
    if addr.is_null() {
        return Err(AsmError::BadAddress);
    }

    let code = unsafe { slice::from_raw_parts(addr, size) };

    disassemble_bytes_ex(code, runtime_addr, instruction_count)
}

pub fn disassemble_bytes(code: &[u8], instruction_count: usize) -> Result<Vec<Inst>, AsmError> {
    disassemble_bytes_ex(code, 0, instruction_count)
}

pub fn disassemble_bytes_ex(
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
