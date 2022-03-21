use crate::restack::{Restack, RestackError};
use crate::elem::{ElemSymbol, ElemType};
use crate::stack::{LineNo};
use crate::types::{TypeId, Context, Type, TypeError};
use crate::types_scratch::{StackInstructionError, Instruction, Instr, Instrs,
    Push, HashSha256, CheckLe, CheckLt, CheckEq,
    StringEq, BytesEq, Concat, Slice, Index, Lookup, AssertTrue, ToJson, UnpackJson, StringToBytes};

// use std::collections::BTreeMap;
// use std::cmp;
// use std::fmt;
// use std::fmt::{Display, Formatter};
// // use std::alloc::string;
// use std::marker::PhantomData;
use std::sync::Arc;
use std::marker::PhantomData;

// use enumset::{EnumSet, enum_set};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Number, Value};
use thiserror::Error;


#[derive(Clone, Debug, Error)]
pub enum InstructionError {
    #[error("Instruction::to_instr UnpackJson does not support: {elem_symbol:?}")]
    UnpackJson {
        elem_symbol: ElemSymbol,
    }
}

impl Instruction {
    pub fn to_instr(self) -> Result<Instr, InstructionError> {
        match self {
            Self::Push(elem) => Ok(Instr::Instr(Arc::new(Push { push: elem }))),
            Self::Restack(restack) => Ok(Instr::Restack(restack.clone())),
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

#[derive(Debug, Error)]
pub enum InstructionTypeError {
    // TODO: move to instruction::
    #[error("Instruction::type_of resulted in an error involving: {instruction:?};\n {error:?}")]
    InstructionTypeOfDetail {
        instruction: Instruction,
        error: Box<Self>,
    },

    // TODO: move to instruction::
    #[error("Instructions::type_of called on an empty Vec of Instruction's")]
    InstructionsTypeOfEmpty,

    // TODO: move to instruction::
    #[error("Instructions::type_of resulted in an error on line: {line_no:?};\n {error:?}")]
    InstructionsTypeOfLineNo {
        line_no: usize,
        error: Box<TypeError>,
    },

    // TODO: move to instruction::
    #[error("Instruction::type_of resulted in restack error: {0:?}")]
    InstructionTypeOfRestack(RestackError),

}

impl Restack {
    // TODO: fix locations: out locations are mislabeled as in locations
    pub fn type_of(&self, line_no: LineNo) -> Result<Type, RestackError> {
        let mut context = Context::new();
        let mut restack_type: Vec<TypeId> = (0..self.restack_depth)
            .map(|x| context.push(ElemType::any(vec![line_no.in_at(x)])))
            .collect();
        let i_type = restack_type.clone();
        self.run(&mut restack_type)?;
        Ok(Type {
            context: context,
            i_type: i_type,
            o_type: restack_type,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub struct Instructions {
    pub instructions: Vec<Instruction>,
}

impl IntoIterator for Instructions {
    type Item = Instruction;
    type IntoIter = <Vec<Instruction> as std::iter::IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.instructions.into_iter()
    }
}


impl Instructions {
    pub fn to_instrs(self) -> Result<Instrs, InstructionError> {
        Ok(Instrs {
            instrs: self.into_iter().map(|x| x.to_instr()).collect::<Result<Vec<Instr>, InstructionError>>()?,
        })
    }
}

impl Instrs {
    pub fn to_instructions(self) -> Result<Instructions, StackInstructionError> {
        Ok(Instructions {
            instructions: self.instrs.into_iter().map(|x| x.to_instruction()).collect::<Result<Vec<Instruction>, StackInstructionError>>()?,
        })
    }
}

// Test program #1: [] -> []
//
// Instruction::Push(Elem::Bool(true)),
// Instruction::Restack(Restack::id()),
// Instruction::AssertTrue,

// Test program #2
//
// ∀ (t0 ∊ {JSON}),
// ∀ (t1 ∊ {JSON}),
// ∀ (t2 ∊ {Object}),
// [t1] ->
// [t0, t2, t1]
//
// Instruction::Push(Elem::Json(Default::default())),
// Instruction::UnpackJson(ElemSymbol::Object),
// Instruction::Restack(Restack::dup()),
