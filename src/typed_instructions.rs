use crate::elem::{Elem, ElemSymbol};
use crate::an_elem::{AnElem};
use crate::types::{Empty, AnError, Nil};
use crate::types_scratch::{AllElems, all_elems_untyped, Singleton, Cons, ReturnSingleton, ConsOut, Or, ReturnOr, IsList};
use crate::untyped_instruction::{Instruction};
use crate::typed_instruction::{IsInstructionT, StackInstructionError};

use std::cmp;
use std::convert::TryFrom;
use std::marker::PhantomData;
use std::fmt::Debug;
use std::sync::Arc;
use std::string::FromUtf8Error;

use enumset::EnumSet;
use generic_array::typenum::{U0, U1, U2};
use serde_json::{Map, Number, Value};
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Concat {}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum ConcatError {}
impl AnError for ConcatError {}

// TODO: add string!
// (Self::String(x), Self::String(y)) => {
//     Ok(Self::String(String::from_utf8(Self::concat_generic(Vec::from(x.clone()), Vec::from(y.clone())))
//                     .map_err(|_| ElemError::ConcatInvalidUTF8 { lhs: x, rhs: y })?))
// },
//
// bytes, array, object
impl IsInstructionT for Concat {
    type IO = ConsOut<ReturnOr<Vec<u8>,             U2,
                      ReturnOr<Vec<Value>,          U2,
               ReturnSingleton<Map<String, Value>,  U2>>>, Nil>;
    type Error = ConcatError;

    fn to_instruction(&self) -> Result<Instruction, StackInstructionError> {
        Ok(Instruction::Concat)
    }

    fn name(_x: PhantomData<Self>) -> String {
        "concat".to_string()
    }

    fn run(&self, x: &Self::IO) -> Result<(), Self::Error> {
        let y = x.clone().hd();
        match y {
            ReturnOr::Left { array, returning } => {
                let lhs = &array[0];
                let rhs = &array[1];
                returning.returning(lhs.into_iter().chain(rhs.into_iter()).cloned().collect());
            },
            ReturnOr::Right(ReturnOr::Left { array, returning }) => {
                let lhs = &array[0];
                let rhs = &array[1];
                returning.returning(lhs.into_iter().chain(rhs.into_iter()).cloned().collect());
            },
            ReturnOr::Right(ReturnOr::Right(ReturnSingleton { singleton, returning })) => {
                let lhs = &singleton.array[0];
                let rhs = &singleton.array[1];
                returning.returning(lhs.into_iter().chain(rhs.into_iter()).map(|xy| (xy.0.clone(), xy.1.clone())).collect());
            },
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AssertTrue {}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
#[error("AssertTrue: found false")]
pub struct AssertTrueError {}
impl AnError for AssertTrueError {}

impl IsInstructionT for AssertTrue {
    type IO = ConsOut<ReturnSingleton<bool, U1>, Nil>;
    type Error = AssertTrueError;

    fn to_instruction(&self) -> Result<Instruction, StackInstructionError> {
        Ok(Instruction::AssertTrue)
    }

    fn name(_x: PhantomData<Self>) -> String {
        "assert_true".to_string()
    }

    fn run(&self, x: &Self::IO) -> Result<(), Self::Error> {
        let array = x.clone().hd().singleton.array;
        let returning = x.clone().hd().returning;
        if array[0] {
            returning.returning(true);
            Ok(())
        } else {
            Err(AssertTrueError {})
        }
    }
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Push<T: AnElem> {
    pub push: T,
}

impl<T: AnElem> IsInstructionT for Push<T> {
    type IO = ConsOut<ReturnSingleton<T, U0>, Nil>;
    type Error = Empty;

    fn to_instruction(&self) -> Result<Instruction, StackInstructionError> {
        Ok(Instruction::Push(self.push.clone().to_elem()))
    }

    fn name(_x: PhantomData<Self>) -> String {
        format!("push_{:?}", AnElem::elem_symbol(PhantomData::<T>))
    }

    fn run(&self, x: &Self::IO) -> Result<(), Self::Error> {
        x.clone().hd().returning.returning(self.push.clone());
        Ok(())
    }
}



#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HashSha256 {}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum HashSha256Error {}
impl AnError for HashSha256Error {}

impl IsInstructionT for HashSha256 {
    type IO = ConsOut<ReturnSingleton<Vec<u8>, U1>, Nil>;
    type Error = Empty;

    fn to_instruction(&self) -> Result<Instruction, StackInstructionError> {
        Ok(Instruction::HashSha256)
    }

    fn name(_x: PhantomData<Self>) -> String {
        "sha256".to_string()
    }

    fn run(&self, x: &Self::IO) -> Result<(), Self::Error> {
        let array = x.clone().hd().singleton.array;
        let returning = x.clone().hd().returning;
        returning.returning(super::sha256(&array[0]));
        Ok(())
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Slice {}

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum SliceError {
    #[error("SliceError::OffsetNotU64: \n{0}")]
    OffsetNotU64(Number),

    #[error("SliceError::LengthNotU64: \n{0}")]
    LengthNotU64(Number),

    #[error("SliceError::Overflow: \noffset: {offset} \nlength: {length}")]
    Overflow {
        offset: Number,
        length: Number,
    },

    #[error("SliceError::TooShort: \noffset: {offset} \nlength: {length} \n{iterable}")]
    TooShort {
        offset: usize,
        length: usize,
        iterable: String,
    },

    #[error("SliceError::FromUtf8Error: \n{0}")]
    FromUtf8Error(FromUtf8Error),
}

impl From<FromUtf8Error> for SliceError {
    fn from(error: FromUtf8Error) -> Self {
        Self::FromUtf8Error(error)
    }
}

impl AnError for SliceError {}

// bytes, string, array, object
impl IsInstructionT for Slice {
    type IO = ConsOut<ReturnOr<Vec<u8>,             U1,
                      ReturnOr<String,              U1,
                      ReturnOr<Vec<Value>,          U1,
               ReturnSingleton<Map<String, Value>,  U1>>>>,
                Cons<Singleton<Number,              U2>, Nil>>;
    type Error = SliceError;

    fn to_instruction(&self) -> Result<Instruction, StackInstructionError> {
        Ok(Instruction::Slice)
    }

    fn name(_x: PhantomData<Self>) -> String {
        "slice".to_string()
    }

    fn run(&self, x: &Self::IO) -> Result<(), Self::Error> {
        let y = x.clone().hd();
        let offset_length = x.clone().tl().hd().array;
        let offset = &offset_length[0];
        let length = &offset_length[1];
        let u_offset = offset.as_u64()
            .ok_or_else(|| SliceError::OffsetNotU64(offset.clone()))
            .and_then(|x| usize::try_from(x).map_err(|_| SliceError::Overflow { offset: offset.clone(), length: length.clone() }))?;
        let u_length = length.as_u64()
            .ok_or_else(|| SliceError::LengthNotU64(length.clone()))
            .and_then(|x| usize::try_from(x).map_err(|_| SliceError::Overflow { offset: offset.clone(), length: length.clone() }))?;
        let u_offset_plus_length = u_offset.checked_add(u_length)
            .ok_or_else(|| SliceError::Overflow { offset: offset.clone(), length: length.clone() })?;
        match y.clone() {
            ReturnOr::Left { array, returning } => {
                let iterable = &array[0];
                if iterable.clone().into_iter().count() < u_offset_plus_length {
                    Err(())
                } else {
                    returning.returning(iterable.into_iter().skip(u_offset).take(u_length).copied().collect());
                    Ok(())
                }
            },
            ReturnOr::Right(ReturnOr::Left { array, returning }) => {
                let iterable = &array[0];
                if iterable.len() < u_offset_plus_length {
                    Err(())
                } else {
                    returning.returning(String::from_utf8(Vec::from(iterable.clone()).into_iter().skip(u_offset).take(u_length).collect())?);
                    Ok(())
                }
            },
            ReturnOr::Right(ReturnOr::Right(ReturnOr::Left { array, returning })) => {
                let iterable = &array[0];
                if iterable.clone().into_iter().count() < u_offset_plus_length {
                    Err(())
                } else {
                    returning.returning(iterable.into_iter().skip(u_offset).take(u_length).cloned().collect());
                    Ok(())
                }
            },
            ReturnOr::Right(ReturnOr::Right(ReturnOr::Right(ReturnSingleton { singleton: Singleton { array }, returning }))) => {
                let iterable = &array[0];
                if iterable.clone().into_iter().count() < u_offset_plus_length {
                    Err(())
                } else {
                    returning.returning(iterable.into_iter().skip(u_offset).take(u_length).map(|xy| (xy.0.clone(), xy.1.clone())).collect());
                    Ok(())
                }
            },
        }.map_err(|_e| {
            SliceError::TooShort {
                offset: u_offset,
                length: u_length,
                // TODO: better error
                iterable: format!("{:?}", y),
            }
        })
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Index {}
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum IndexError {
    #[error("Index: index not valid u64: {0:?}")]
    IndexNotU64(Number),

    #[error("Index: index not valid usize: {0:?}")]
    Overflow(Number),

    #[error("Index: iterable: {iterable:?}\nis too short for index: {index:?}")]
    TooShort {
        index: usize,
        iterable: String,
    },
}
impl AnError for IndexError {}

// bytes, array, object
impl IsInstructionT for Index {
    type IO = ConsOut<ReturnSingleton<Value,                U0>,
                       Cons<Singleton<Number,               U1>,
                              Cons<Or<Vec<Value>,           U1,
                            Singleton<Map<String, Value>,   U1>>, Nil>>>;
    type Error = IndexError;

    fn to_instruction(&self) -> Result<Instruction, StackInstructionError> {
        Ok(Instruction::Index)
    }

    fn name(_x: PhantomData<Self>) -> String {
        "index".to_string()
    }

    fn run(&self, x: &Self::IO) -> Result<(), Self::Error> {
        let returning = x.clone().hd().returning;
        let index = x.clone().tl().hd().array[0].clone();
        let y = &x.clone().tl().tl().hd();
        let u_index = index.as_u64()
            .ok_or_else(|| IndexError::IndexNotU64(index.clone()))
            .and_then(|x| usize::try_from(x).map_err(|_| IndexError::Overflow(index.clone())))?;

        let result = match y.clone() {
            Or::Left(array) => {
                array[0]
                    .clone()
                    .into_iter()
                    .skip(u_index)
                    .next()
            },
            Or::Right(Singleton { array }) => {
                array[0]
                    .clone()
                    .into_iter()
                    .skip(u_index)
                    .next()
                    .map(|(_x, y)| y)
            },
        }.ok_or_else(|| {
            IndexError::TooShort {
                index: u_index,
                // TODO: better error
                iterable: format!("{:?}", y),
            }
        })?;
        returning.returning(result);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToJson {}
#[derive(Clone, Debug, Error)]
#[error("ToJson failed with a serde_json error: \n{input} \n{error}")]
pub struct ToJsonError {
    input: Elem,
    error: Arc<serde_json::Error>,
}
impl AnError for ToJsonError {}

impl IsInstructionT for ToJson {
    type IO = ConsOut<ReturnSingleton<Value, U0>, Cons<AllElems<U1>, Nil>>;
    type Error = ToJsonError;

    fn to_instruction(&self) -> Result<Instruction, StackInstructionError> {
        Ok(Instruction::ToJson)
    }

    fn name(_x: PhantomData<Self>) -> String {
        "to_json".to_string()
    }

    fn run(&self, x: &Self::IO) -> Result<(), Self::Error> {
        let returning = x.clone().hd().returning;
        let y = &x.clone().tl().hd();
        let array = all_elems_untyped(y);
        let z = array[0].clone();
        returning.returning(serde_json::to_value(z.clone())
                            .map_err(move |e| ToJsonError {
                                input: z,
                                error: Arc::new(e),
        })?);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lookup {}
#[derive(Clone, Debug, PartialEq, Eq, Error)]
#[error("Lookup failed, key not in map: \n{key:?} \n{map:?}")]
pub struct LookupError {
    key: String,
    map: Map<String, Value>,
}
impl AnError for LookupError {}

impl IsInstructionT for Lookup {
    type IO = ConsOut<ReturnSingleton<Value, U0>,
                 Cons<Singleton<String, U1>,
                 Cons<Singleton<Map<String, Value>, U1>, Nil>>>;
    type Error = LookupError;

    fn to_instruction(&self) -> Result<Instruction, StackInstructionError> {
        Ok(Instruction::Lookup)
    }

    fn name(_x: PhantomData<Self>) -> String {
        "lookup".to_string()
    }

    fn run(&self, x: &Self::IO) -> Result<(), Self::Error> {
        let returning = x.clone().hd().returning;
        let key = &x.clone().tl().hd().array[0];
        let map = &x.clone().tl().tl().hd().array[0];
        returning.returning(map.get(key)
           .ok_or_else(|| LookupError {
               key: key.clone(),
               map: map.clone(),
           })?.clone());
        Ok(())
    }
}


#[derive(Debug)]
pub struct UnpackJson<T: AnElem> {
    pub t: PhantomData<T>,
}
#[derive(Debug, Error)]
#[error("UnpackJson failed to unpack JSON: \n{elem_symbol:?} \n{input}")]
pub struct UnpackJsonError {
    elem_symbol: EnumSet<ElemSymbol>,
    input: Value,
}
impl AnError for UnpackJsonError {}

pub trait AJsonElem: AnElem {
    fn to_value(self) -> Value;
    fn from_value(t: PhantomData<Self>, x: Value) -> Option<Self> where Self: Sized;
}

impl AJsonElem for () {
    fn to_value(self) -> Value {
        Value::Null
    }

    fn from_value(_t: PhantomData<Self>, x: Value) -> Option<Self> where Self: Sized {
        match x {
            Value::Null => Some(()),
            _ => None,
        }
    }
}

impl AJsonElem for bool {
    fn to_value(self) -> Value {
        Value::Bool(self)
    }

    fn from_value(_t: PhantomData<Self>, x: Value) -> Option<Self> where Self: Sized {
        match x {
            Value::Bool(y) => Some(y),
            _ => None,
        }
    }
}

impl AJsonElem for Number {
    fn to_value(self) -> Value {
        Value::Number(self)
    }

    fn from_value(_t: PhantomData<Self>, x: Value) -> Option<Self> where Self: Sized {
        match x {
            Value::Number(y) => Some(y),
            _ => None,
        }
    }
}

impl AJsonElem for String {
    fn to_value(self) -> Value {
        Value::String(self)
    }

    fn from_value(_t: PhantomData<Self>, x: Value) -> Option<Self> where Self: Sized {
        match x {
            Value::String(y) => Some(y),
            _ => None,
        }
    }
}

impl AJsonElem for Vec<Value> {
    fn to_value(self) -> Value {
        Value::Array(self)
    }

    fn from_value(_t: PhantomData<Self>, x: Value) -> Option<Self> where Self: Sized {
        match x {
            Value::Array(y) => Some(y),
            _ => None,
        }
    }
}

impl AJsonElem for Map<String, Value> {
    fn to_value(self) -> Value {
        Value::Object(self)
    }

    fn from_value(_t: PhantomData<Self>, x: Value) -> Option<Self> where Self: Sized {
        match x {
            Value::Object(y) => Some(y),
            _ => None,
        }
    }
}

impl<T: AJsonElem> IsInstructionT for UnpackJson<T> {
    type IO = ConsOut<ReturnSingleton<T, U0>,
                       Cons<Singleton<Value, U1>, Nil>>;
    type Error = UnpackJsonError;

    fn to_instruction(&self) -> Result<Instruction, StackInstructionError> {
        let mut symbol_set = <T as AnElem>::elem_symbol(PhantomData).into_iter();
        match (symbol_set.next(), symbol_set.next()) {
            (Some(elem_symbol), None) => Ok(Instruction::UnpackJson(elem_symbol)),
            (x, y) => Err(StackInstructionError::UnpackJsonNotSingleton {
                first_value: x,
                second_value: y,
            }),
        }
    }

    fn name(_x: PhantomData<Self>) -> String {
        "unpack_json".to_string()
    }

    fn run(&self, x: &Self::IO) -> Result<(), Self::Error> {
        let returning = x.clone().hd().returning;
        let json = &x.clone().tl().hd().array[0];
        let result =
            AJsonElem::from_value(PhantomData::<T>, json.clone())
            .ok_or_else(|| UnpackJsonError {
                elem_symbol: AnElem::elem_symbol(PhantomData::<T>),
                input: json.clone(),
            })?;
        returning.returning(result);
        Ok(())
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StringToBytes {}

impl IsInstructionT for StringToBytes {
    type IO = ConsOut<ReturnSingleton<Vec<u8>, U0>, Cons<Singleton<String, U1>, Nil>>;
    type Error = Empty;

    fn to_instruction(&self) -> Result<Instruction, StackInstructionError> {
        Ok(Instruction::StringToBytes)
    }

    fn name(_x: PhantomData<Self>) -> String {
        "string_to_bytes".to_string()
    }

    fn run(&self, x: &Self::IO) -> Result<(), Self::Error> {
        let returning = x.clone().hd().returning;
        let in_str = &x.clone().tl().hd().array[0];
        returning.returning(in_str.clone().into_bytes());
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CheckLe {}
#[derive(Clone, Debug, PartialEq, Eq, Error)]
#[error("CheckLe applied to incomparable elements: \n{lhs:?}\n {rhs:?}\n")]
pub struct CheckLeError {
    lhs: Elem,
    rhs: Elem,
}
impl AnError for CheckLeError {}

impl IsInstructionT for CheckLe {
    type IO = ConsOut<ReturnSingleton<bool, U0>, Cons<AllElems<U2>, Nil>>;
    type Error = CheckLeError;

    fn to_instruction(&self) -> Result<Instruction, StackInstructionError> {
        Ok(Instruction::CheckLe)
    }

    fn name(_x: PhantomData<Self>) -> String {
        "check_le".to_string()
    }

    fn run(&self, x: &Self::IO) -> Result<(), Self::Error> {
        let returning = x.clone().hd().returning;
        let y = &x.clone().tl().hd();
        let array = all_elems_untyped(y);
        let lhs = array[0].clone();
        let rhs = array[1].clone();
        let cmp_result = lhs.partial_cmp(&rhs)
            .ok_or_else(|| CheckLeError {
                lhs: lhs,
                rhs: rhs
        })?;
        let result = match cmp_result {
            cmp::Ordering::Less => true,
            cmp::Ordering::Equal => true,
            cmp::Ordering::Greater => false,
        };
        returning.returning(result);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CheckLt {}
#[derive(Clone, Debug, PartialEq, Eq, Error)]
#[error("CheckLt applied to incomparable elements: \n{lhs:?}\n {rhs:?}\n")]
pub struct CheckLtError {
    lhs: Elem,
    rhs: Elem,
}
impl AnError for CheckLtError {}

impl IsInstructionT for CheckLt {
    type IO = ConsOut<ReturnSingleton<bool, U0>, Cons<AllElems<U2>, Nil>>;
    type Error = CheckLtError;

    fn to_instruction(&self) -> Result<Instruction, StackInstructionError> {
        Ok(Instruction::CheckLt)
    }

    fn name(_x: PhantomData<Self>) -> String {
        "check_lt".to_string()
    }

    fn run(&self, x: &Self::IO) -> Result<(), Self::Error> {
        let returning = x.clone().hd().returning;
        let y = &x.clone().tl().hd();
        let array = all_elems_untyped(y);
        let lhs = array[0].clone();
        let rhs = array[1].clone();
        let cmp_result = lhs.partial_cmp(&rhs)
            .ok_or_else(|| CheckLtError {
                lhs: lhs,
                rhs: rhs
        })?;
        let result = match cmp_result {
            cmp::Ordering::Less => true,
            _ => false,
        };
        returning.returning(result);
        Ok(())
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CheckEq {}
#[derive(Clone, Debug, PartialEq, Eq, Error)]
#[error("CheckEq applied to incomparable elements: \n{lhs:?}\n {rhs:?}\n")]
pub struct CheckEqError {
    lhs: Elem,
    rhs: Elem,
}
impl AnError for CheckEqError {}

impl IsInstructionT for CheckEq {
    type IO = ConsOut<ReturnSingleton<bool, U0>, Cons<AllElems<U2>, Nil>>;
    type Error = CheckEqError;

    fn to_instruction(&self) -> Result<Instruction, StackInstructionError> {
        Ok(Instruction::CheckEq)
    }

    fn name(_x: PhantomData<Self>) -> String {
        "check_eq".to_string()
    }

    fn run(&self, x: &Self::IO) -> Result<(), Self::Error> {
        let returning = x.clone().hd().returning;
        let y = &x.clone().tl().hd();
        let array = all_elems_untyped(y);
        let lhs = array[0].clone();
        let rhs = array[1].clone();
        let cmp_result = lhs.partial_cmp(&rhs)
            .ok_or_else(|| CheckEqError {
                lhs: lhs,
                rhs: rhs
        })?;
        let result = match cmp_result {
            cmp::Ordering::Equal => true,
            _ => false,
        };
        returning.returning(result);
        Ok(())
    }
}

/// input: [x: String, y: String]
/// output: [x == y: bool]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StringEq {}

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum StringEqError {}
impl AnError for StringEqError {}

impl IsInstructionT for StringEq {
    type IO = ConsOut<ReturnSingleton<bool, U0>, Cons<Singleton<String, U2>, Nil>>;
    type Error = StringEqError;

    fn to_instruction(&self) -> Result<Instruction, StackInstructionError> {
        Ok(Instruction::StringEq)
    }

    fn name(_x: PhantomData<Self>) -> String {
        "check_eq".to_string()
    }

    fn run(&self, x: &Self::IO) -> Result<(), Self::Error> {
        let returning = x.clone().hd().returning;
        let array = &x.clone().tl().hd().array;
        let lhs = array[0].clone();
        let rhs = array[1].clone();
        returning.returning(lhs == rhs);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BytesEq {}
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum BytesEqError {}
impl AnError for BytesEqError {}

impl IsInstructionT for BytesEq {
    type IO = ConsOut<ReturnSingleton<bool, U0>, Cons<Singleton<Vec<u8>, U2>, Nil>>;
    type Error = BytesEqError;

    fn to_instruction(&self) -> Result<Instruction, StackInstructionError> {
        Ok(Instruction::BytesEq)
    }

    fn name(_x: PhantomData<Self>) -> String {
        "check_eq".to_string()
    }

    fn run(&self, x: &Self::IO) -> Result<(), Self::Error> {
        let returning = x.clone().hd().returning;
        let array = &x.clone().tl().hd().array;
        let lhs = array[0].clone();
        let rhs = array[1].clone();
        returning.returning(lhs == rhs);
        Ok(())
    }
}

