use crate::types::{Elem, Instruction, Instructions, Restack, RestackError};

use thiserror::Error;

// TODO: implement n-step executor && errors that tell you what step they're on
// to allow minimal step-by-step debugging
#[derive(Debug, Default)]
pub struct Executor {
    stack: Vec<Elem>,
}

impl Executor {
    pub fn consume(&mut self, expressions: Instructions) -> Result<(), ExecError> {
        for expr in expressions {
            match expr {
                Instruction::Push(elem) => self.push(elem),
                Instruction::FnRestack(restack) => self.restack(restack)?,
                Instruction::FnAssertTrue => self.assert_true()?,
                Instruction::FnCheckLe => self.check_le()?,
                Instruction::FnCheckLt => self.check_lt()?,
                Instruction::FnCheckEqual => self.check_equal()?,
                Instruction::FnConcat => self.concat()?,
                Instruction::FnSlice => self.slice()?,
                Instruction::FnIndex => self.index()?,
                Instruction::FnLookup => self.lookup()?,
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

    fn check_le(&mut self) -> Result<(), ExecError> {
        let one = self.pop()?;
        let other = self.pop()?;
        self.push(Elem::Bool(one <= other));
        Ok(())
    }

    fn check_lt(&mut self) -> Result<(), ExecError> {
        let one = self.pop()?;
        let other = self.pop()?;
        self.push(Elem::Bool(one < other));
        Ok(())
    }

    fn check_equal(&mut self) -> Result<(), ExecError> {
        let one = self.pop()?;
        let other = self.pop()?;
        self.push(Elem::Bool(one == other));
        Ok(())
    }

    fn concat(&mut self) -> Result<(), ExecError> {
        let one = self.pop()?;
        let other = self.pop()?;
        match (one, other) {
            (Elem::Bytes(x), Elem::Bytes(y)) => {
                let mut result = x.clone();
                result.append(&mut y.clone());
                self.push(Elem::Bytes(result));
                Ok(()) },

            (Elem::String(x), Elem::String(y)) => {
                let mut result = x.clone();
                result.push_str(&mut y.clone());
                self.push(Elem::String(result));
                Ok(()) },

            (Elem::Array(x), Elem::Array(y)) => {
                let mut result = x.clone();
                result.append(&mut y.clone());
                self.push(Elem::Array(result));
                Ok(()) },

            (Elem::Object(x), Elem::Object(y)) => {
                let mut result = x.clone();
                result.append(&mut y.clone());
                self.push(Elem::Object(result));
                Ok(()) },

            (some_x, some_y) => {
                let lhs = &some_x.simple_type();
                let rhs = &some_y.simple_type();
                Err(ExecError::ConcatUnsupportedTypes { lhs: lhs, rhs: rhs }) },
        }
    }

    fn slice(&mut self) -> Result<(), ExecError> {
        let one = self.pop()?;
        // let other = self.pop()?;
        // self.push(Elem::Bool(one == other));
        Ok(())
    }

    fn index(&mut self) -> Result<(), ExecError> {
        let one = self.pop()?;
        // let other = self.pop()?;
        // self.push(Elem::Bool(one == other));
        Ok(())
    }

    fn lookup(&mut self) -> Result<(), ExecError> {
        let one = self.pop()?;
        // let other = self.pop()?;
        // self.push(Elem::Bool(one == other));
        Ok(())
    }

    fn sha256(&mut self) -> Result<(), ExecError> {
        match self.pop()? {
            Elem::Bytes(bytes) => {
                self.push(Elem::Bytes(super::sha256(&bytes)));
                Ok(())
            }
            elem => Err(ExecError::HashUnsupportedType(elem.simple_type())),
        }
    }

    fn push(&mut self, elem: Elem) {
        self.stack.push(elem)
    }

    // TODO: since pop can fail, require passing debug info to it
    fn pop(&mut self) -> Result<Elem, ExecError> {
        self.stack.pop().ok_or_else(|| ExecError::EmptyStack)
    }

    fn restack(&mut self, restack: Restack) -> Result<(), ExecError> {
        match restack.run(&mut self.stack) {
            Err(e) => Err(ExecError::RestackExecError(e)),
            Ok(new_stack) => {
                self.stack = new_stack;
                Ok(()) },
        }
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
    #[error("restack failed: {0}")]
    RestackExecError(RestackError),
    #[error("concat applied to unsupported types: lhs: {lhs:?}; rhs: {rhs:?}")]
    ConcatUnsupportedTypes {
        lhs: &'static str,
        rhs: &'static str,
    },
}
