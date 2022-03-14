use crate::restack::{Restack, RestackError};
use crate::elem::{Elem, ElemError};
/* use crate::types::{Instruction, Instructions}; */
use crate::instruction::{Instruction, Instructions};
use thiserror::Error;

#[derive(Debug, Default)]
pub struct Executor {
    stack: Vec<Elem>,
}

impl Executor {
    pub fn consume(&mut self, expressions: Instructions) -> Result<(), ExecError> {
        self.debug()?;
        for expr in expressions {
            println!("------------------------------------------------------------------------------------------");
            println!("#: {:?}", expr);
            match expr {
                Instruction::Restack(restack) => self.restack(restack)?,
                Instruction::Push(elem) => self.push(elem),

                Instruction::AssertTrue => self.pop()?.assert_true()?,

                Instruction::HashSha256 => self.pop_push(Elem::sha256)?,
                Instruction::ToJson => self.pop_push(Elem::to_json)?,
                Instruction::UnpackJson(elem_symbol) => self.pop_push(|x| x.unpack_json(elem_symbol))?,
                Instruction::StringToBytes => self.pop_push(Elem::string_to_bytes)?,

                Instruction::CheckLe => self.pop_push2(Elem::check_le)?,
                Instruction::CheckLt => self.pop_push2(Elem::check_lt)?,
                Instruction::CheckEq => self.pop_push2(Elem::check_eq)?,
                Instruction::Concat => self.pop_push2(Elem::concat)?,
                Instruction::Index =>  self.pop_push2(Elem::index)?,
                Instruction::Lookup => self.pop_push2(Elem::lookup)?,

                Instruction::Slice => self.pop_push3(Elem::slice)?,
            }
            self.debug()?;
        }
        Ok(())
    }

    pub fn debug(&self) -> Result<(), ExecError> {
        println!("------------------------------------------------------------------------------------------");
        for stack_elem in &self.stack {
            println!("------------------------------");
            println!("{}", serde_json::to_string_pretty(stack_elem)?)
        }
        Ok(())
    }

    fn restack(&mut self, restack: Restack) -> Result<(), ExecError> {
        restack.run(&mut self.stack)?;
        Ok(())
    }

    // TODO: since pop can fail, require passing debug info to it
    // (so we know what we were expecting)
    fn pop(&mut self) -> Result<Elem, ExecError> {
        let result = self.stack.get(0).ok_or_else(|| ExecError::EmptyStack).map(|x|x.clone())?;
        self.stack = self.stack.drain(1..).collect();
        Ok(result.clone())
    }

    pub fn push(&mut self, elem: Elem) {
        let mut memo = vec![elem];
        memo.append(&mut self.stack.clone());
        self.stack = memo;
    }

    pub fn pop_push(&mut self, f: impl Fn(Elem) -> Result<Elem, ElemError>) -> Result<(), ExecError> {
        let one = self.pop()?;
        self.push(f(one)?);
        Ok(())
    }

    pub fn pop_push2(&mut self, f: impl Fn(Elem, Elem) -> Result<Elem, ElemError>) -> Result<(), ExecError> {
        let one = self.pop()?;
        let other = self.pop()?;
        self.push(f(one, other)?);
        Ok(())
    }

    pub fn pop_push3(&mut self, f: impl Fn(Elem, Elem, Elem) -> Result<Elem, ElemError>) -> Result<(), ExecError> {
        let first = self.pop()?;
        let second = self.pop()?;
        let third = self.pop()?;
        self.push(f(first, second, third)?);
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ExecError {
    #[error("ElemError({0:?})")]
    ElemError(ElemError),
    #[error("tried to pop from an empty stack")]
    EmptyStack,
    #[error("restack failed: {0}")]
    RestackExecError(RestackError),
}

impl From<ElemError> for ExecError {
    fn from(error: ElemError) -> Self {
        ExecError::ElemError(error)
    }
}

impl From<serde_json::Error> for ExecError {
    fn from(error: serde_json::Error) -> Self {
        ExecError::ElemError(ElemError::ToFromJsonFailed(format!("{}", error)))
    }
}

impl From<RestackError> for ExecError {
    fn from (error: RestackError) -> Self {
        ExecError::RestackExecError(error)
    }
}
