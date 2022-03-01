// use crate::restack::{RestackError};
use crate::elem::{Elem, ElemSymbol};

// use std::collections::BTreeMap;
// use std::cmp;
// use std::iter::{FromIterator};

// use std::fmt;
// use std::fmt::{Display, Formatter};
// // use std::alloc::string;
// use std::marker::PhantomData;
// use std::sync::Arc;

// use enumset::{EnumSet, enum_set};
use serde::{Deserialize, Serialize};
// use serde_json::{Map, Number, Value};
use thiserror::Error;


// TODO: use for execution
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub struct Stack {
    stack: Vec<Elem>,
}

impl Stack {
    // TODO: since pop can fail, require passing debug info to it
    // (so we know what we were expecting)
    pub fn pop(&mut self) -> Result<Elem, StackError> {
        let result = self.stack.get(0).ok_or_else(|| StackError::EmptyStack).map(|x|x.clone())?;
        self.stack = self.stack.drain(1..).collect();
        Ok(result.clone())
    }

    pub fn push(&mut self, elem: Elem) {
        let mut memo = vec![elem];
        // memo.append(&mut self.stack.clone());
        memo.append(&mut self.stack);
        self.stack = memo;
    }
}

#[derive(Clone, Debug, Error)]
pub enum StackError {
    #[error("Stack::pop: tried to pop from an empty stack")]
    EmptyStack,

    #[error("HList::pop: element popped from the stack {found:?} wasn't the expected type {expected:?} (remaining stack: {stack:?})")]
    UnexpectedElemType {
        expected: ElemSymbol,
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
