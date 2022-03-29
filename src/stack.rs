use crate::elem::{Elem, ElemSymbol};
use crate::elem_type::{StackType};
use crate::an_elem::{AnElem, AnElemError};
use crate::location::{LineNo};

use std::fmt;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

use enumset::{EnumSet};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use generic_array::{GenericArray, ArrayLength};
use typenum::marker_traits::Unsigned;

// TODO: pub field needed?
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub struct Stack {
    pub stack: Vec<Elem>,
}

impl Display for Stack {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_list()
            .entries(self.stack
                     .iter()
                     .map(|x| format!("{}", x)))
            .finish()?;
        Ok(())
    }
}


impl Stack {
    pub fn new() -> Self {
        Stack {
            stack: vec![],
        }
    }

    // TODO: since pop can fail, require passing debug info to it
    // (so we know what we were expecting)
    pub fn pop(&mut self) -> Result<Elem, StackError> {
        let result = self.stack.get(0).ok_or_else(|| StackError::EmptyStack).map(|x|x.clone())?;
        self.stack = self.stack.drain(1..).collect();
        Ok(result.clone())
    }

    pub fn pop_elem<T: AnElem>(&mut self, _t: PhantomData<T>) -> Result<T, StackError> {
        let hd_elem = self.pop()?;
        Ok(<T as AnElem>::from_elem(PhantomData, hd_elem)?)
    }

    pub fn push(&mut self, elem: Elem) {
        let mut memo = vec![elem];
        memo.append(&mut self.stack);
        self.stack = memo;
    }

    pub fn push_elem(&mut self, elem: impl AnElem) {
        self.push(elem.to_elem())
    }

    // TODO: reversed?
    pub fn pop_generic_array<T: AnElem, N: ArrayLength<T>>(&mut self,
                                                           _t: PhantomData<T>,
                                                           _n: PhantomData<N>) -> Result<GenericArray<T, N>, StackError> {
        let mut xs = vec![];
        for _current_index in 1..<N as Unsigned>::USIZE {
            let hd_elem = self.pop()?;
            xs.push(AnElem::from_elem(PhantomData::<T>, hd_elem)?)
        }
        GenericArray::from_exact_iter(xs).ok_or_else(|| StackError::TODO)
    }

    pub fn type_of(&self) -> StackType {
        StackType {
            types: self.stack.clone().into_iter().map(|x| x.elem_type(vec![])).collect(),
        }
    }

    pub fn debug_type(&self) -> () {
        println!("stack type:\n{}", self.type_of())
    }

    pub fn debug(&self) -> Result<(), serde_json::Error> {
        self.debug_type();
        println!("------------------------------------------------------------------------------------------");
        for stack_elem in &self.stack {
            println!("------------------------------");
            println!("{}", serde_json::to_string_pretty(stack_elem)?)
        }
        Ok(())
    }
}


#[derive(Clone, Debug, Error)]
pub enum StackError {
    #[error("Stack::pop: tried to pop from an empty stack")]
    EmptyStack,

    #[error("Stack:pop_elem threw an error from AnElem\n{0}")]
    AnElemError(AnElemError),

    #[error("pop: element popped from the stack {found:?} wasn't the expected type {expected:?} (remaining stack: {stack})")]
    UnexpectedElemTypeIn {
        expected: EnumSet<ElemSymbol>,
        found: Elem,
        stack: Stack,
    },

    #[error("Stack::run_instruction: instruction {name:?} produced error: {error:?}\non line number: {line_no:?}")]
    RunInstruction {
        name: String,
        error: String,
        line_no: LineNo,
    },

    #[error("Stack::pop_generic_array: unimplemented")]
    TODO,
}

impl From<AnElemError> for StackError {
    fn from(x: AnElemError) -> Self {
        Self::AnElemError(x)
    }
}
