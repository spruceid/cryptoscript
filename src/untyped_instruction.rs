use crate::elem::{Elem, ElemSymbol};
use crate::restack::{Restack};

use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub enum Instruction {
    Push(Elem),
    Restack(Restack),
    HashSha256,
    CheckLe,
    CheckLt,
    CheckEq,
    StringEq,
    BytesEq,
    Concat,
    Slice,
    Index,
    Lookup,
    AssertTrue,
    ToJson,
    UnpackJson(ElemSymbol),
    StringToBytes,
}

#[derive(Clone, Debug, Error)]
pub enum InstructionError {
    #[error("Instruction::to_instr UnpackJson does not support: {elem_symbol:?}")]
    UnpackJson {
        elem_symbol: ElemSymbol,
    }
}

