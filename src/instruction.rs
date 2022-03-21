use crate::restack::{Restack, RestackError};
use crate::elem::{ElemSymbol, ElemType};
use crate::stack::{LineNo};
use crate::types::{TypeId, Context, Type, TypeError};
use crate::types_scratch::{StackInstructionError, Instruction, Instr, Instrs,
    Push, HashSha256, CheckLe, CheckLt, CheckEq,
    StringEq, Concat, Slice, Index, Lookup, AssertTrue, ToJson, UnpackJson, StringToBytes};

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


// #[derive(Clone, Debug)]
// pub struct Instrs {
//     // TODO: replace Result with Either?
//     pub instrs: Vec<Result<Arc<dyn IsStackInstruction>, Restack>>,
// }



// fn example_instrs() -> Instrs {
//     Instrs {
//         instrs: vec![
//             Arc::new(Concat {}),
//             Arc::new(AssertTrue {}),
//             Arc::new(Push { push: () }),
//             Arc::new(HashSha256 {}),
//             Arc::new(Slice {}),
//             Arc::new(Index {}),
//             Arc::new(ToJson {}),
//             Arc::new(Lookup {}),
//             Arc::new(UnpackJson { t: PhantomData::<()> }),
//             Arc::new(StringToBytes {}),
//             Arc::new(CheckLe {}),
//             Arc::new(CheckLt {}),
//             Arc::new(CheckEq {})
//         ],
//     }
// }



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


// /// Push(Elem),             // (t: type, elem: type(t)) : [] -> [ t ]
// /// Restack(Restack),       // (r: restack) : [ .. ] -> [ .. ]
// /// HashSha256,             // : [ bytes ] -> [ bytes ]
// /// CheckLe,                // : [ x, x ] -> [ bool ]
// /// CheckLt,                // : [ x, x ] -> [ bool ]
// /// CheckEq,                // : [ x, x ] -> [ bool ]
// /// Concat,                 // (t: type, prf: is_concat(t)) : [ t, t ] -> [ t ]
// /// Slice,                  // (t: type, prf: is_slice(t)) : [ int, int, t ] -> [ t ]
// /// Index,                  // (t: type, prf: is_index(t)) : [ int, t ] -> [ json ]
// /// Lookup,                 // [ string, object ] -> [ json ]
// /// AssertTrue,             // [ bool ] -> []
// /// ToJson,                 // (t: type) : [ t ] -> [ json ]
// /// UnpackJson(ElemSymbol), // (t: type) : [ json ] -> [ t ]
// /// StringToBytes,          // [ string ] -> [ bytes ]
// impl Instruction {
//     pub fn type_of(&self, line_no: LineNo) -> Result<Type, InstructionTypeError> {
//         match self {
//             Instruction::Restack(restack) =>
//                 Ok(restack
//                    .type_of(line_no)
//                    .or_else(|e| Err(InstructionTypeError::InstructionTypeOfRestack(e)))?),

//             Instruction::AssertTrue => {
//                 let mut context = Context::new();
//                 let bool_var = context
//                     .push(ElemSymbol::Bool
//                           .elem_type(vec![line_no.in_at(0)]));
//                 Ok(Type {
//                     context: context,
//                     i_type: vec![bool_var],
//                     o_type: vec![],
//                 })
//             },

//             Instruction::Push(elem) => {
//                 let mut context = Context::new();
//                 let elem_var = context
//                     .push(elem.elem_type(vec![line_no.out_at(0)]));
//                 Ok(Type {
//                     context: context,
//                     i_type: vec![],
//                     o_type: vec![elem_var],
//                 })
//             },

//             Instruction::HashSha256 => {
//                 let mut context = Context::new();
//                 let bytes_var = context.push(ElemSymbol::Bytes.elem_type(vec![line_no.in_at(0), line_no.out_at(0)]));
//                 Ok(Type {
//                     context: context,
//                     i_type: vec![bytes_var],
//                     o_type: vec![bytes_var],
//                 })
//             },

//             Instruction::ToJson => {
//                 let mut context = Context::new();
//                 let any_var = context.push(ElemType::any(vec![line_no.in_at(0)]));
//                 let json_var = context.push(ElemSymbol::Json.elem_type(vec![line_no.out_at(0)]));
//                 Ok(Type {
//                     context: context,
//                     i_type: vec![any_var],
//                     o_type: vec![json_var],
//                 })
//             },

//             Instruction::StringToBytes => {
//                 let mut context = Context::new();
//                 let string_var = context.push(ElemSymbol::String.elem_type(vec![line_no.in_at(0)]));
//                 let bytes_var = context.push(ElemSymbol::Bytes.elem_type(vec![line_no.out_at(0)]));
//                 Ok(Type {
//                     context: context,
//                     i_type: vec![string_var],
//                     o_type: vec![bytes_var],
//                 })
//             },

//             Instruction::UnpackJson(elem_symbol) => {
//                 let mut context = Context::new();
//                 let json_var = context.push(ElemSymbol::Json.elem_type(vec![line_no.in_at(0)]));
//                 let elem_symbol_var = context.push(elem_symbol.elem_type(vec![line_no.out_at(0)]));
//                 Ok(Type {
//                     context: context,
//                     i_type: vec![json_var],
//                     o_type: vec![elem_symbol_var],
//                 })
//             },

//             Instruction::CheckLe => {
//                 let mut context = Context::new();
//                 let any_lhs_var = context.push(ElemType::any(vec![line_no.in_at(0)]));
//                 let any_rhs_var = context.push(ElemType::any(vec![line_no.in_at(1)]));
//                 let bool_var = context.push(ElemSymbol::Bool.elem_type(vec![line_no.out_at(0)]));
//                 Ok(Type {
//                     context: context,
//                     i_type: vec![any_lhs_var, any_rhs_var],
//                     o_type: vec![bool_var],
//                 })
//             },

//             Instruction::CheckLt => {
//                 let mut context = Context::new();
//                 let any_lhs_var = context.push(ElemType::any(vec![line_no.in_at(0)]));
//                 let any_rhs_var = context.push(ElemType::any(vec![line_no.in_at(1)]));
//                 let bool_var = context.push(ElemSymbol::Bool.elem_type(vec![line_no.out_at(0)]));
//                 Ok(Type {
//                     context: context,
//                     i_type: vec![any_lhs_var, any_rhs_var],
//                     o_type: vec![bool_var],
//                 })
//             },

//             Instruction::CheckEq => {
//                 let mut context = Context::new();
//                 let any_lhs_var = context.push(ElemType::any(vec![line_no.in_at(0)]));
//                 let any_rhs_var = context.push(ElemType::any(vec![line_no.in_at(1)]));
//                 let bool_var = context.push(ElemSymbol::Bool.elem_type(vec![line_no.out_at(0)]));
//                 Ok(Type {
//                     context: context,
//                     i_type: vec![any_lhs_var, any_rhs_var],
//                     o_type: vec![bool_var],
//                 })
//             },

//             Instruction::Concat => {
//                 let mut context = Context::new();
//                 let concat_var = context.push(ElemType::concat_type(vec![line_no.in_at(0), line_no.in_at(1), line_no.out_at(0)]));
//                 Ok(Type {
//                     context: context,
//                     i_type: vec![concat_var, concat_var],
//                     o_type: vec![concat_var],
//                 })
//             },

//             Instruction::Index => {
//                 let mut context = Context::new();
//                 let number_var = context.push(ElemSymbol::Number.elem_type(vec![line_no.in_at(0)]));
//                 let index_var = context.push(ElemType::index_type(vec![line_no.in_at(1), line_no.out_at(0)]));
//                 Ok(Type {
//                     context: context,
//                     i_type: vec![number_var, index_var],
//                     o_type: vec![index_var],
//                 })
//             },

//             Instruction::Lookup => {
//                 let mut context = Context::new();
//                 let string_var = context.push(ElemSymbol::String.elem_type(vec![line_no.in_at(0)]));
//                 let object_var = context.push(ElemSymbol::Object.elem_type(vec![line_no.in_at(1), line_no.out_at(0)]));
//                 Ok(Type {
//                     context: context,
//                     i_type: vec![string_var, object_var],
//                     o_type: vec![object_var],
//                 })
//             },

//             Instruction::Slice => {
//                 let mut context = Context::new();
//                 let offset_number_var = context.push(ElemSymbol::Number.elem_type(vec![line_no.in_at(0)]));
//                 let length_number_var = context.push(ElemSymbol::Number.elem_type(vec![line_no.in_at(1)]));
//                 let slice_var = context.push(ElemType::slice_type(vec![line_no.in_at(2), line_no.out_at(0)]));
//                 Ok(Type {
//                     context: context,
//                     i_type: vec![offset_number_var, length_number_var, slice_var],
//                     o_type: vec![slice_var],
//                 })
//             },
//         }.or_else(|e| Err(InstructionTypeError::InstructionTypeOfDetail {
//             instruction: self.clone(),
//             error: Box::new(e),
//         }))
//     }
// }


// impl Instructions {
//     pub fn type_of(&self) -> Result<Type, InstructionTypeError> {
//         let mut current_type = Type::id();
//         for (i, instruction) in self.instructions.iter().enumerate() {
//             current_type = current_type.compose(instruction.type_of(From::from(i + 1))?)
//                 .or_else(|e| Err(InstructionTypeError::InstructionsTypeOfLineNo { // TODO: deprecated by Location
//                     line_no: i,
//                     error: Box::new(e),
//                 }))?;

//             println!("line {i}: {current_type}", i = i, current_type = current_type);
//         }
//         Ok(current_type)
//     }
// }

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
