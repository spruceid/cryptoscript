use crate::restack::{Restack, RestackError};
use crate::types::{Elem, ElemSymbol, ElemError, Instruction, Instructions};
use thiserror::Error;

// TODO: implement n-step executor && errors that tell you what step they're on
// to allow minimal step-by-step debugging
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
                Instruction::Push(elem) => self.push(elem),
                Instruction::Restack(restack) => self.restack(restack)?,
                Instruction::AssertTrue => self.assert_true()?,
                Instruction::CheckLe => self.check_le()?,
                Instruction::CheckLt => self.check_lt()?,
                Instruction::CheckEq => self.check_eq()?,
                Instruction::Concat => self.concat()?,
                Instruction::Slice => self.slice()?,
                Instruction::Index => self.index()?,
                Instruction::Lookup => self.lookup()?,
                Instruction::HashSha256 => self.sha256()?,
                Instruction::ToJson => self.to_json()?,
                Instruction::FromJson => self.from_json()?,
                Instruction::UnpackJson(elem_symbol) => self.unpack_json(elem_symbol)?,
                Instruction::StringToBytes => self.string_to_bytes()?,
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

    pub fn push(&mut self, elem: Elem) {
        let mut memo = vec![elem];
        memo.append(&mut self.stack.clone());
        self.stack = memo;
    }

    fn restack(&mut self, restack: Restack) -> Result<(), ExecError> {
        match restack.run(&mut self.stack) {
            Err(e) => Err(ExecError::RestackExecError(e)),
            Ok(new_stack) => {
                self.stack = new_stack;
                Ok(()) },
        }
    }

    fn assert_true(&mut self) -> Result<(), ExecError> {
        let x = self.pop()?;
        x.assert_true()?;
        Ok(())
    }

    fn check_le(&mut self) -> Result<(), ExecError> {
        let one = self.pop()?;
        let other = self.pop()?;
        self.push(one.check_le(other)?);
        Ok(())
    }

    fn check_lt(&mut self) -> Result<(), ExecError> {
        let one = self.pop()?;
        let other = self.pop()?;
        self.push(one.check_lt(other)?);
        Ok(())
    }

    fn check_eq(&mut self) -> Result<(), ExecError> {
        let one = self.pop()?;
        let other = self.pop()?;
        self.push(one.check_eq(other)?);
        Ok(())
    }

    fn concat(&mut self) -> Result<(), ExecError> {
        let one = self.pop()?;
        let other = self.pop()?;
        self.push(one.concat(other)?);
        Ok(())
    }

    fn slice(&mut self) -> Result<(), ExecError> {
        let maybe_offset = self.pop()?;
        let maybe_length = self.pop()?;
        let maybe_iterable = self.pop()?;
        self.push(Elem::slice(maybe_offset, maybe_length, maybe_iterable)?);
        Ok(())
    }

    // you can index any iterable
    fn index(&mut self) -> Result<(), ExecError> {
        let maybe_index = self.pop()?;
        let maybe_iterable = self.pop()?;
        self.push(Elem::index(maybe_index, maybe_iterable)?);
        Ok(())
    }

    // you can lookup a key in a Map (or fail, no recovery)
    fn lookup(&mut self) -> Result<(), ExecError> {
        let maybe_key = self.pop()?;
        let maybe_map = self.pop()?;
        self.push(Elem::lookup(maybe_key, maybe_map)?);
        Ok(())
    }

    fn sha256(&mut self) -> Result<(), ExecError> {
        let hash_input = self.pop()?;
        self.push(Elem::sha256(hash_input)?);
        Ok(())
    }

    fn to_json(&mut self) -> Result<(), ExecError> {
        let to_json_input = self.pop()?;
        self.push(Elem::to_json(to_json_input)?);
        Ok(())
    }

    fn from_json(&mut self) -> Result<(), ExecError> {
        let from_json_input = self.pop()?;
        self.push(Elem::from_json(from_json_input)?);
        Ok(())
    }

    fn unpack_json(&mut self, elem_symbol: ElemSymbol) -> Result<(), ExecError> {
        let unpack_json_input = self.pop()?;
        self.push(Elem::unpack_json(unpack_json_input, elem_symbol)?);
        Ok(())
    }

    fn string_to_bytes(&mut self) -> Result<(), ExecError> {
        let string_to_bytes_input = self.pop()?;
        self.push(Elem::string_to_bytes(string_to_bytes_input)?);
        Ok(())
    }

    // TODO: since pop can fail, require passing debug info to it
    // (so we know what we were expecting)
    fn pop(&mut self) -> Result<Elem, ExecError> {
        // self.stack.pop().ok_or_else(|| ExecError::EmptyStack)
        let result = self.stack.get(0).ok_or_else(|| ExecError::EmptyStack).map(|x|x.clone())?;
        self.stack = self.stack.drain(1..).collect();
        Ok(result.clone())
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

