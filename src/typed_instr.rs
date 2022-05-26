use crate::elem::ElemSymbol;
use crate::restack::Restack;
use crate::untyped_instruction::{Instruction, InstructionError};
use crate::typed_instruction::{IsStackInstruction, StackInstructionError};
use crate::typed_instructions::{AssertTrue, Lookup, Concat, Slice, Push,
    StringEq, BytesEq, ToJson, Index, CheckLe, CheckLt, CheckEq, HashSha256,
    StringToBytes, UnpackJson};

use std::marker::PhantomData;
use std::fmt::Debug;
use std::sync::Arc;

use serde_json::{Map, Number, Value};

/// A dynamically-resolved IsStackInstruction or Restack
#[derive(Clone, Debug)]
pub enum Instr {
    /// Dynamically-resolved IsStackInstruction
    Instr(Arc<dyn IsStackInstruction>),

    /// Restack
    Restack(Restack),
}

impl Instr {
    /// Convert an Instr (typed) to an Instruction (untyped)
    pub fn to_instruction(&self) -> Result<Instruction, StackInstructionError> {
        match self {
            Self::Instr(instr) => instr.to_instruction(),
            Self::Restack(restack) => Ok(Instruction::Restack(restack.clone())),
        }
    }
}

impl Instruction {
    /// Convert an Instruction to an Instr, only failing when UnpackJson is
    /// applied to an ElemSymbol that doesn't represent valid JSON
    pub fn to_instr(self) -> Result<Instr, InstructionError> {
        match self {
            Self::Push(elem) => Ok(Instr::Instr(Arc::new(Push { push: elem }))),
            Self::Restack(restack) => Ok(Instr::Restack(restack)),
            Self::HashSha256 => Ok(Instr::Instr(Arc::new(HashSha256 {}))),
            Self::CheckLe => Ok(Instr::Instr(Arc::new(CheckLe {}))),
            Self::CheckLt => Ok(Instr::Instr(Arc::new(CheckLt {}))),
            Self::CheckEq => Ok(Instr::Instr(Arc::new(CheckEq {}))),
            Self::StringEq => Ok(Instr::Instr(Arc::new(StringEq {}))),
            Self::BytesEq => Ok(Instr::Instr(Arc::new(BytesEq {}))),
            Self::Concat => Ok(Instr::Instr(Arc::new(Concat {}))),
            Self::Slice => Ok(Instr::Instr(Arc::new(Slice {}))),
            Self::Index => Ok(Instr::Instr(Arc::new(Index {}))),
            Self::Lookup => Ok(Instr::Instr(Arc::new(Lookup {}))),
            Self::AssertTrue => Ok(Instr::Instr(Arc::new(AssertTrue {}))),
            Self::ToJson => Ok(Instr::Instr(Arc::new(ToJson {}))),
            Self::UnpackJson(elem_symbol) => {
                match elem_symbol {
                    ElemSymbol::Unit => Ok(Instr::Instr(Arc::new(UnpackJson { t: PhantomData::<()> }))),
                    ElemSymbol::Bool => Ok(Instr::Instr(Arc::new(UnpackJson { t: PhantomData::<bool> }))),
                    ElemSymbol::Number => Ok(Instr::Instr(Arc::new(UnpackJson { t: PhantomData::<Number> }))),
                    ElemSymbol::String => Ok(Instr::Instr(Arc::new(UnpackJson { t: PhantomData::<String> }))),
                    ElemSymbol::Array => Ok(Instr::Instr(Arc::new(UnpackJson { t: PhantomData::<Vec<Value>> }))),
                    ElemSymbol::Object => Ok(Instr::Instr(Arc::new(UnpackJson { t: PhantomData::<Map<String, Value>> }))),
                    _ => Err(InstructionError::UnpackJson {
                        elem_symbol: elem_symbol,
                    })
                }
            },
            Self::StringToBytes => Ok(Instr::Instr(Arc::new(StringToBytes {}))),
        }
    }
}

