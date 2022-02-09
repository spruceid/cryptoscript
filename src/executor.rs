use crate::types::{Elem, Instruction, Instructions, Restack, RestackError};
use std::convert::TryFrom;

use serde_json::{Map, Number, Value};
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

    fn concat_bytes(&mut self, x: Vec<u8>, y: Vec<u8>) -> Result<(), ExecError> {
        let mut result = x.clone();
        result.append(&mut y.clone());
        self.push(Elem::Bytes(result));
        Ok(())
    }

    fn concat_string(&mut self, x: String, y: String) -> Result<(), ExecError> {
        let mut result = x.clone();
        result.push_str(&mut y.clone());
        self.push(Elem::String(result));
        Ok(())
    }

    fn concat_array(&mut self, x: Vec<Elem>, y: Vec<Elem>) -> Result<(), ExecError> {
        let mut result = x.clone();
        result.append(&mut y.clone());
        self.push(Elem::Array(result));
        Ok(())
    }

    fn concat_object(&mut self, x: Map<String, Value>, y: Map<String, Value>) -> Result<(), ExecError> {
        let mut result = x.clone();
        result.append(&mut y.clone());
        self.push(Elem::Object(result));
        Ok(())
    }

    fn concat(&mut self) -> Result<(), ExecError> {
        let one = self.pop()?;
        let other = self.pop()?;
        match (one, other) {
            (Elem::Bytes(x), Elem::Bytes(y)) => self.concat_bytes(x, y),
            (Elem::String(x), Elem::String(y)) => self.concat_string(x, y),
            (Elem::Array(x), Elem::Array(y)) => self.concat_array(x, y),
            (Elem::Object(x), Elem::Object(y)) => self.concat_object(x, y),
            (some_x, some_y) => {
                Err(ExecError::ConcatUnsupportedTypes {
                    lhs: &some_x.simple_type(),
                    rhs: &some_y.simple_type()
                })
            },
        }
    }


    fn slice_bytes(&mut self, offset: Number, length: Number, iterable: Vec<u8>) -> Result<(), ExecError> {
        let u_offset = offset.as_u64()
            .ok_or_else(|| ExecError::SliceOffsetNotU64(offset.clone()))
            .and_then(|x| usize::try_from(x).map_err(|_| ExecError::SliceOverflow { offset: offset.clone(), length: length.clone() }))?;
        let u_length = length.as_u64()
            .ok_or_else(|| ExecError::SliceLengthNotU64(length.clone()))
            .and_then(|x| usize::try_from(x).map_err(|_| ExecError::SliceOverflow { offset: offset.clone(), length: length.clone() }))?;
        let u_offset_plus_length = u_offset.checked_add(u_length)
            .ok_or_else(|| ExecError::SliceOverflow { offset: offset.clone(), length: length.clone() })?;
        if iterable.len() < u_offset_plus_length {
            Err(ExecError::SliceTooShort {
                offset: u_offset,
                length: u_length,
                iterable: &Elem::Bytes(vec![]).simple_type(),
            })
        } else {
            self.push(Elem::Bytes(iterable[u_offset..=u_offset_plus_length].to_vec()));
            Ok(())
        }
    }

    fn slice_string(&mut self, offset: Number, length: Number, iterable: String) -> Result<(), ExecError> {
        let result = iterable.clone();
        // result.push_str(&mut y.clone());
        // self.push(Elem::String(result));
        Ok(())
    }

    fn slice_array(&mut self, offset: Number, length: Number, iterable: Vec<Elem>) -> Result<(), ExecError> {
        let result = iterable.clone();
        // result.append(&mut y.clone());
        // self.push(Elem::Array(result));
        Ok(())
    }

    fn slice_object(&mut self, offset: Number, length: Number, iterable: Map<String, Value>) -> Result<(), ExecError> {
        let result = iterable.clone();
        // result.append(&mut y.clone());
        // self.push(Elem::Object(result));
        Ok(())
    }

    // slice : offset -> length -> iterable -> iterable
    fn slice(&mut self) -> Result<(), ExecError> {
        let maybe_offset = self.pop()?;
        let maybe_length = self.pop()?;
        let maybe_iterable = self.pop()?;
        match (maybe_offset, maybe_length, maybe_iterable) {
            (Elem::Number(offset), Elem::Number(length), Elem::Bytes(iterator)) =>
                self.slice_bytes(offset, length, iterator),
            (Elem::Number(offset), Elem::Number(length), Elem::String(iterator)) =>
                self.slice_string(offset, length, iterator),
            (Elem::Number(offset), Elem::Number(length), Elem::Array(iterator)) =>
                self.slice_array(offset, length, iterator),
            (Elem::Number(offset), Elem::Number(length), Elem::Object(iterator)) =>
                self.slice_object(offset, length, iterator),
            (maybe_not_offset, maybe_not_length, maybe_not_iterable) => {
                Err(ExecError::SliceUnsupportedTypes {
                    maybe_not_offset: &maybe_not_offset.simple_type(),
                    maybe_not_length: &maybe_not_length.simple_type(),
                    maybe_not_iterable: &maybe_not_iterable.simple_type(),
                })
            }
        }
    }


    // you can index any iterable
    fn index(&mut self) -> Result<(), ExecError> {
        let one = self.pop()?;
        // let other = self.pop()?;
        // self.push(Elem::Bool(one == other));
        Ok(())
    }



    // TODO:
    // - lookup_null : key -> map -> Result<(), ExecError>
    // - lookup_bool : key -> map -> Result<bool, ExecError>
    // - lookup_number : .. -> Result<Number, ..>
    // - lookup_string : .. -> Result<String, ..>
    // - lookup_array : .. -> Result<Vec<Value>, ..>
    // - lookup_object : .. -> Result<Map<String, Value>, ..>

    // you can lookup a key in a Map (or fail, no recovery)
    fn lookup(&mut self) -> Result<(), ExecError> {
        let maybe_key = self.pop()?;
        let maybe_map = self.pop()?;
        // match (maybe_key, maybe_map) {
        //     (Elem::String(key), Elem::Object(map)) => {
        //         match map.get(key) {
        //             Some(value) => {
        //                 self.push()

        //                 Ok(()) }
        //         },

        //     (maybe_not_key, maybe_not_map) => Err(ExecError::LookupUnsupportedTypes {
        //         maybe_not_key: &maybe_not_key.simple_type(),
        //         maybe_not_map: &maybe_not_map.simple_type(),
        //     })
        // }

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

    #[error("slice applied to unsupported types: maybe_not_offset: {maybe_not_offset:?}; maybe_not_length: {maybe_not_length:?}; maybe_not_iterable: {maybe_not_iterable:?}")]
    SliceUnsupportedTypes {
        maybe_not_offset: &'static str,
        maybe_not_length: &'static str,
        maybe_not_iterable: &'static str,
    },
    #[error("slice applied to an 'offset' that can't be unpacked to u64: offset: {0:?}")]
    SliceOffsetNotU64(Number),
    #[error("slice applied to a 'length' that can't be unpacked to u64: length: {0:?}")]
    SliceLengthNotU64(Number),
    #[error("slice applied to an iterable that's too short for the given offset: offset: {offset:?} and length: {length:?}: iterable: {iterable:?}")]
    SliceTooShort {
        offset: usize,
        length: usize,
        iterable: &'static str,
    },
    #[error("slice applied to offset and length whose sum overflows usize: offset: {offset:?} and length: {length:?}")]
    SliceOverflow {
        offset: Number,
        length: Number,
    },

    #[error("lookup applied to unsupported types: maybe_not_key: {maybe_not_key:?}; maybe_not_map: {maybe_not_map:?}")]
    LookupUnsupportedTypes {
        maybe_not_key: &'static str,
        maybe_not_map: &'static str,
    },
}
