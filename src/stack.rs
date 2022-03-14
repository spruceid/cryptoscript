// use crate::restack::{RestackError};
use crate::elem::{Elem, AnElem, AnElemError, ElemSymbol};

// use std::collections::BTreeMap;
// use std::cmp;
// use std::iter::{FromIterator};

// use std::fmt;
// use std::fmt::{Display, Formatter};
// // use std::alloc::string;
// use std::marker::PhantomData;
// use std::sync::Arc;
use std::marker::PhantomData;

use enumset::{EnumSet};
use serde::{Deserialize, Serialize};
// use serde_json::{Map, Number, Value};
use thiserror::Error;
use generic_array::{GenericArray, ArrayLength};
use typenum::marker_traits::Unsigned;


// TODO: use for execution
// TODO: pub field needed?
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub struct Stack {
    pub stack: Vec<Elem>,
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
        // memo.append(&mut self.stack.clone());
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
}



#[derive(Clone, Debug, Error)]
pub enum StackError {
    #[error("Stack::pop: tried to pop from an empty stack")]
    EmptyStack,

    #[error("Stack:pop_elem threw an error from AnElem {0:?}")]
    AnElemError(AnElemError),

    #[error("pop: element popped from the stack {found:?} wasn't the expected type {expected:?} (remaining stack: {stack:?})")]
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

// TODO: relocate LineNo, ArgumentIndex, Location
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LineNo {
    pub line_no: usize,
}

impl From<usize> for LineNo {
    fn from(line_no: usize) -> Self {
        LineNo {
            line_no: line_no,
        }
    }
}

pub type ArgumentIndex = usize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Location {
    line_no: LineNo,
    argument_index: ArgumentIndex,
    is_input: bool,
}

impl LineNo {
    pub fn in_at(&self, argument_index: usize) -> Location {
        Location {
            line_no: *self,
            argument_index: argument_index,
            is_input: true,
        }
    }

    pub fn out_at(&self, argument_index: usize) -> Location {
        Location {
            line_no: *self,
            argument_index: argument_index,
            is_input: false,
        }
    }
}
