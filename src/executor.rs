use crate::types::{Elem, Instruction, Instructions};

use thiserror::Error;

#[derive(Debug, Default)]
pub struct Executor {
    stack: Vec<Elem>,
}

impl Executor {
    pub fn consume(&mut self, expressions: Instructions) -> Result<(), ExecError> {
        for expr in expressions {
            match expr {
                Instruction::Push(elem) => self.push(elem),
                Instruction::FnAssertTrue => self.assert_true()?,
                Instruction::FnCheckEqual => self.check_equal()?,
                Instruction::FnHashSha256 => self.sha256()?,
            }
        }
        Ok(())
    }

    fn assert_true(&mut self) -> Result<(), ExecError> {
        match self.pop()? {
            Elem::Bool(true) => Ok(()),
            found => Err(ExecError::AssertTrueFailed(found)),
        }
    }

    fn check_equal(&mut self) -> Result<(), ExecError> {
        let one = self.pop()?;
        let other = self.pop()?;
        self.push(Elem::Bool(one == other));
        Ok(())
    }

    fn sha256(&mut self) -> Result<(), ExecError> {
        match self.pop()? {
            Elem::BytesN(bytes) => {
                self.push(Elem::Bytes32(super::sha256(&bytes)));
                Ok(())
            }
            elem => Err(ExecError::HashUnsupportedType(elem.simple_type())),
        }
    }

    fn push(&mut self, elem: Elem) {
        self.stack.push(elem)
    }

    fn pop(&mut self) -> Result<Elem, ExecError> {
        self.stack.pop().ok_or_else(|| ExecError::EmptyStack)
    }
}

#[derive(Debug, Error)]
pub enum ExecError {
    #[error("expected Elem::Bool(true), found {0:?}")]
    AssertTrueFailed(Elem),
    #[error("tried to pop from an empty stack")]
    EmptyStack,
    #[error("attempted to hash an elem of an unsupported type ({0})")]
    HashUnsupportedType(&'static str),
}
