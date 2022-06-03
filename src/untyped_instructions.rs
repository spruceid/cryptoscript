use crate::untyped_instruction::{Instruction, InstructionError};
use crate::typed_instruction::StackInstructionError;
use crate::typed_instr::Instr;
use crate::typed_instrs::Instrs;

use serde::{Deserialize, Serialize};

/// A list of untyped instructions
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub struct Instructions {
    /// A list of untyped instructions
    pub instructions: Vec<Instruction>,
}

impl IntoIterator for Instructions {
    type Item = Instruction;
    type IntoIter = <Vec<Instruction> as std::iter::IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.instructions.into_iter()
    }
}

impl Default for Instructions {
    fn default() -> Self {
        Self::new()
    }
}

impl Instructions {
    /// New empty list of untyped instructions
    pub fn new() -> Self {
        Instructions {
            instructions: vec![],
        }
    }

    /// Push an Instruction onto the end of the list of instructions
    pub fn push(&mut self, instruction: Instruction) {
        self.instructions.push(instruction)
    }

    /// Convert to a list of typed instructions
    pub fn to_instrs(self) -> Result<Instrs, InstructionError> {
        Ok(Instrs {
            instrs: self.into_iter().map(|x| x.to_instr()).collect::<Result<Vec<Instr>, InstructionError>>()?,
        })
    }
}

impl Instrs {
    /// Convert to a list of untyped instructions
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
