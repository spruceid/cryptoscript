use crate::elem::ElemSymbol;
use crate::stack::Stack;
use crate::restack::RestackError;
use crate::types::{Type, TypeError};
use crate::elems::ElemsPopError;
use crate::elems_list::IsList;
use crate::elems_list_input_output::IOList;
use crate::untyped_instruction::Instruction;

use std::marker::PhantomData;
use std::fmt::Debug;
use std::sync::Arc;

use thiserror::Error;

/// A typed instruction with explicit input, output, and error types
pub trait IsInstructionT: Debug {
    /// The input/output type of the instruction
    type IO: IOList;

    /// All possible errors that can result from running this instruction.
    /// Empty can be used for none.
    type Error: std::error::Error;

    /// Convert to an untyped instruction
    fn to_instruction(&self) -> Result<Instruction, StackInstructionError>;

    /// The String name of the Instruction
    fn name(x: PhantomData<Self>) -> String;

    /// Run the instruction, returning all results using the IOList interface
    fn run(&self, x: &Self::IO) -> Result<(), Self::Error>;
}

#[derive(Clone, Debug, Error)]
pub enum StackInstructionError {
    #[error("StackInstructionError::ElemsPopError:\n{0}")]
    ElemsPopError(ElemsPopError),

    #[error("RawStackInstructionError:\n{0}")]
    RawStackInstructionErrorString(String),

    #[error("MissingOutput:\n{instruction}\n\n{stack_input}")]
    // TODO: more granular error typing
    MissingOutput {
        instruction: String,
        stack_input: String,
    },

    #[error("Instrs::type_of_mono type error:\n{0}")]
    TypeError(TypeError),

    #[error("StackInstructionError::RestackError:\n{0}")]
    RestackError(RestackError),

    #[error("StackInstructionError::DebugJsonError:\n{0}")]
    DebugJsonError(Arc<serde_json::Error>),

    #[error("UnpackJsonNotSingleton:\n{first_value:?}\n{second_value:?}")]
    UnpackJsonNotSingleton {
        first_value: Option<ElemSymbol>,
        second_value: Option<ElemSymbol>,
    },

}

pub trait IsStackInstruction: Debug {
    fn to_instruction(&self) -> Result<Instruction, StackInstructionError>;
    fn name(&self) -> String;
    fn type_of(&self) -> Result<Type, ElemsPopError>;
    fn stack_run(&self, stack: &mut Stack) -> Result<(), StackInstructionError>;
}

impl<T> IsStackInstruction for T
where
    T: IsInstructionT,
{
    fn to_instruction(&self) -> Result<Instruction, StackInstructionError> {
        self.to_instruction()
    }

    fn name(&self) -> String {
        IsInstructionT::name(PhantomData::<Self>)
    }

    fn type_of(&self) -> Result<Type, ElemsPopError> {
        IOList::type_of(PhantomData::<<T as IsInstructionT>::IO>)
    }

    fn stack_run(&self, stack: &mut Stack) -> Result<(), StackInstructionError> {
        let stack_input = &IsList::pop(PhantomData::<<T as IsInstructionT>::IO>, stack)
            .map_err(StackInstructionError::ElemsPopError)?;
        self.run(stack_input)
            .map_err(|e| StackInstructionError::RawStackInstructionErrorString(format!("{:?}", e)))?;
        let output_value = stack_input
            .returning()
            .ok_or_else(|| StackInstructionError::MissingOutput {
                instruction: format!("{:?}", self),
                stack_input: format!("{:?}", stack_input),
            })?;
        stack.push(output_value);
        Ok(())
    }
}

