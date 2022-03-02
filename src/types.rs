// use crate::restack::{RestackError};
use crate::elem::{AnElem, Elem, ElemSymbol};
use crate::stack::{Stack, StackError, LineNo, Location};

use std::collections::BTreeMap;
use std::cmp;
use std::iter::{FromIterator};

use std::fmt;
use std::fmt::{Display, Formatter};
// use std::alloc::string;
use std::marker::PhantomData;
// use std::sync::Arc;

use enumset::{EnumSet, enum_set};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Number, Value};
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Empty {}

impl Empty {
    pub fn absurd<T>(&self, _p: PhantomData<T>) -> T {
        match *self {}
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Nil { }

impl Iterator for Nil {
    type Item = Elem;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

pub trait ElemList: Clone + IntoIterator<Item = Elem> {
    type Hd: AnElem;
    type Tl: ElemList;

    fn is_empty(&self) -> bool;
    fn hd(&self) -> Self::Hd;
    fn tl(&self) -> Self::Tl;
    fn cons<T: AnElem>(self, x: T) -> ConsElem<T, Self> where Self: Sized;
    fn pop(x: PhantomData<Self>, stack: &mut Stack) -> Result<Self, StackError>;
}

impl ElemList for Nil {
    type Hd = ();
    type Tl = Nil;

    fn is_empty(&self) -> bool {
        true
    }

    fn hd(&self) -> Self::Hd {
        ()
    }

    fn tl(&self) -> Self::Tl {
        Self {}
    }

    fn cons<T: AnElem>(self, x: T) -> ConsElem<T, Self>
    where
        Self: Sized,
    {
        ConsElem {
            hd: x,
            tl: self,
        }
    }

    fn pop(_x: PhantomData<Self>, _stack: &mut Stack) -> Result<Self, StackError> {
        Ok(Nil {})
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct ConsElem<T: AnElem, U: ElemList> {
    hd: T,
    tl: U,
}

#[derive(Clone, PartialEq, Eq)]
pub struct IterConsElem<T: AnElem, U: ElemList> {
    cons: ConsElem<T, U>,
    at_head: bool,
}

impl<T: AnElem, U: ElemList> IntoIterator for ConsElem<T, U> {
    type Item = Elem;
    type IntoIter = IterConsElem<T, U>;

    fn into_iter(self) -> Self::IntoIter {
        IterConsElem {
            cons: self,
            at_head: true,
        }
    }
}

impl<T: AnElem, U: ElemList> Iterator for IterConsElem<T, U> {
    type Item = Elem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.at_head {
            Some(self.cons.clone().hd.to_elem())
        } else {
            let self_cons = self.cons.clone();
            *self = self_cons.into_iter();
            self.next()
        }
    }
}

impl<T: AnElem, U: ElemList> ElemList for ConsElem<T, U> {
    type Hd = T;
    type Tl = U;

    fn is_empty(&self) -> bool {
        false
    }

    fn hd(&self) -> Self::Hd {
        self.hd.clone()
    }

    fn tl(&self) -> Self::Tl {
        self.tl.clone()
    }

    fn cons<V: AnElem>(self, x: V) -> ConsElem<V, Self>
    where
        Self: Sized,
    {
        ConsElem {
            hd: x,
            tl: self,
        }
    }

    // TODO: add better errors
    fn pop(_x: PhantomData<Self>, stack: &mut Stack) -> Result<Self, StackError> {
        let hd_elem = stack.pop()?;
        Ok(ConsElem {
            hd: <Self::Hd as AnElem>::from_elem(PhantomData, hd_elem)?,
            tl: Self::Tl::pop(PhantomData, stack)?,
        })
    }
}


pub trait FromElemList {
    type AsElemList: ElemList;
    fn to_hlist_type(x: PhantomData<Self>) -> PhantomData<Self::AsElemList>;
    // fn to_hlist(&self) -> Self::AsElemList;
    fn from_hlist(t: PhantomData<Self>, x: Self::AsElemList) -> Self;
}

impl FromElemList for () {
    type AsElemList = Nil;
    fn to_hlist_type(_x: PhantomData<Self>) -> PhantomData<Self::AsElemList> { PhantomData }
    fn from_hlist(_t: PhantomData<Self>, _x: Self::AsElemList) -> Self { () }
}

impl<T, U> FromElemList for (T, U)
where
    T: AnElem,
    U: AnElem,
{
    type AsElemList = ConsElem<T, ConsElem<U, Nil>>;
    fn to_hlist_type(_x: PhantomData<Self>) -> PhantomData<Self::AsElemList> { PhantomData }
    // fn to_hlist(&self) -> Self::AsElemList {
    //     Nil.cons(self.0).cons(self.1)
    // }

    fn from_hlist(_t: PhantomData<Self>, x: Self::AsElemList) -> Self {
        (x.hd(), x.tl().hd())
    }
}

impl<T, U> FromElemList for ConsElem<T, U>
where
    T: AnElem,
    U: ElemList,
{
    type AsElemList = Self;
    fn to_hlist_type(_x: PhantomData<Self>) -> PhantomData<Self::AsElemList> { PhantomData }
    // fn to_hlist(&self) -> Self::AsElemList {
    //     Nil.cons(self.0).cons(self.1)
    // }

    fn from_hlist(_t: PhantomData<Self>, x: Self::AsElemList) -> Self { x }
}



// TODO: add necessary traits, methods and rename to IsInstructionError or IsInstrError
pub trait AnError: std::fmt::Debug {
    // fn to_stack_error(&self, line_no: LineNo) -> StackError;
}

impl StackError {
    fn instruction_default(name: &str, error_str: &str, line_no: LineNo) -> Self {
        Self::RunInstruction {
            name: name.to_string(),
            error: error_str.to_string(),
            line_no: line_no,
        }
    }
}

impl AnError for Empty {
    // fn to_stack_error(&self, _line_no: LineNo) -> StackError {
    //     self.absurd(PhantomData)
    // }
}

// TODO: add Default, etc
pub trait IsInstruction: std::fmt::Debug {
    type In: FromElemList;
    type Out: AnElem;
    type Error: AnError;

    fn run(&self, x: Self::In) -> Result<Self::Out, Self::Error>;
}

impl Stack {
    fn run_instruction<T: IsInstruction>(&mut self, instr: T, line_no: LineNo) -> Result<(), StackError> {
        let input = ElemList::pop(FromElemList::to_hlist_type(PhantomData::<T::In>), self)?;
        let output = instr.run(FromElemList::from_hlist(PhantomData::<T::In>, input)).map_err(|e| StackError::RunInstruction {
            name: format!("{:?}", instr),
            error: format!("{:?}", e),
            line_no: line_no})?;
        Ok(self.push(output.to_elem()))
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AssertTrue {}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AssertTrueError {}
impl AnError for AssertTrueError {}

impl IsInstruction for AssertTrue {
    type In = ConsElem<bool, Nil>;
    type Out = bool;
    type Error = AssertTrueError;

    fn run(&self, x: Self::In) -> Result<Self::Out, Self::Error> {
        if x.hd() {
            Ok(x.hd())
        } else {
            Err(AssertTrueError {})
        }
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Push<T: AnElem> {
    push: T,
}

impl<T: AnElem> IsInstruction for Push<T> {
    type In = ();
    type Out = T;
    type Error = Empty;

    fn run(&self, _x: Self::In) -> Result<Self::Out, Self::Error> {
        Ok(self.push.clone())
    }
}





#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HashSha256 {}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HashSha256Error {}
impl AnError for HashSha256Error {}

impl IsInstruction for HashSha256 {
    type In = ConsElem<Vec<u8>, Nil>;
    type Out = Vec<u8>;
    type Error = Empty;

    fn run(&self, x: Self::In) -> Result<Self::Out, Self::Error> {
        Ok(super::sha256(&x.hd()))
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Concat<T: AnElem> {
    t: PhantomData<T>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ConcatError {}
impl AnError for ConcatError {}

impl<T: AnElem + IntoIterator + FromIterator<<T as IntoIterator>::Item>> IsInstruction for Concat<T> {
    type In = (T, T);
    type Out = T;
    type Error = Empty;

    fn run(&self, x: Self::In) -> Result<Self::Out, Self::Error> {
        let (lhs, rhs) = x;
        Ok(lhs.into_iter().chain(rhs.into_iter()).collect())
    }
}

    // pub fn concat(self, other: Self) -> Result<Self, ElemError> {
    //     match (self, other) {
    //         (Self::Bytes(x), Self::Bytes(y)) => Ok(Self::Bytes(Self::concat_generic(x, y))),
    //         (Self::String(x), Self::String(y)) => {
    //             Ok(Self::String(String::from_utf8(Self::concat_generic(Vec::from(x.clone()), Vec::from(y.clone())))
    //                             .map_err(|_| ElemError::ConcatInvalidUTF8 { lhs: x, rhs: y })?))
    //         },
    //         (Self::Array(x), Self::Array(y)) => Ok(Self::Array(Self::concat_generic(x, y))),
    //         (Self::Object(x), Self::Object(y)) => Ok(Self::Object(Self::concat_generic(x, y))),
    //         (some_x, some_y) => {
    //             Err(ElemError::ConcatUnsupportedTypes {
    //                 lhs: some_x.symbol_str(),
    //                 rhs: some_y.symbol_str()
    //             })
    //         },
    //     }
    // }



#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Slice {}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SliceError {}
impl AnError for SliceError {}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Index {}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct IndexError {}
impl AnError for IndexError {}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Lookup {}
#[derive(Clone, Debug, PartialEq, Eq)]
struct LookupError {
    key: String,
    map: Map<String, Value>,
}
impl AnError for LookupError {}

impl IsInstruction for Lookup {
    type In = (String, Map<String, Value>);
    type Out = Value;
    type Error = LookupError;

    fn run(&self, x: Self::In) -> Result<Self::Out, Self::Error> {
        let (key, map) = x;
        Ok(map.get(&key)
           .ok_or_else(|| LookupError {
               key: key,
               map: map.clone(),
           })?.clone())
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct UnpackJson<T: AnElem> {
    t: PhantomData<T>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct UnpackJsonError {}
impl AnError for UnpackJsonError {}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct StringToBytes {}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct StringToBytesError {}
impl AnError for StringToBytesError {}



// TODO: POLYMORPHIC W/ ANY: PERHAPS Elem: AnElem ??
//
// ideas:
// 1. use macros when it's a trait
// 2. gradual typing: allow Elem to be AnElem


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CheckLe {}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CheckLeError {}
impl AnError for CheckLeError {}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CheckLt {}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CheckLtError {}
impl AnError for CheckLtError {}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CheckEq {}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CheckEqError {}
impl AnError for CheckEqError {}





    // pub fn check_le(self, other: Self) -> Result<Self, ElemError> {
    //     let result = match self.partial_cmp(&other)
    //         .ok_or_else(|| ElemError::CheckLeIncomparableTypes {
    //             lhs: self.symbol_str(),
    //             rhs: other.symbol_str() })? {
    //                 cmp::Ordering::Less => true,
    //                 cmp::Ordering::Equal => true,
    //                 cmp::Ordering::Greater => false,
    //     };
    //     Ok(Self::Bool(result))
    // }

    // pub fn check_lt(self, other: Self) -> Result<Self, ElemError> {
    //     let result = match self.partial_cmp(&other)
    //         .ok_or_else(|| ElemError::CheckLtIncomparableTypes {
    //             lhs: self.symbol_str(),
    //             rhs: other.symbol_str() })? {
    //                 cmp::Ordering::Less => true,
    //                 _ => false,
    //     };
    //     Ok(Self::Bool(result))
    // }

    // pub fn check_eq(self, other: Self) -> Result<Self, ElemError> {
    //     let result = match self.partial_cmp(&other)
    //         .ok_or_else(|| ElemError::CheckEqIncomparableTypes {
    //             lhs: self.symbol_str(),
    //             rhs: other.symbol_str() })? {
    //                 cmp::Ordering::Equal => true,
    //                 _ => false,
    //     };
    //     Ok(Self::Bool(result))
    // }

    // fn slice_generic<T: Clone + IntoIterator +
    //   std::iter::FromIterator<<T as std::iter::IntoIterator>::Item>>(offset: Number,
    //                                                                  length: Number,
    //                                                                  iterable: T,
    //                                                                  elem_symbol: ElemSymbol) ->
    //     Result<T, ElemError> {
    //     let u_offset = offset.as_u64()
    //         .ok_or_else(|| ElemError::SliceOffsetNotU64(offset.clone()))
    //         .and_then(|x| usize::try_from(x).map_err(|_| ElemError::SliceOverflow { offset: offset.clone(), length: length.clone() }))?;
    //     let u_length = length.as_u64()
    //         .ok_or_else(|| ElemError::SliceLengthNotU64(length.clone()))
    //         .and_then(|x| usize::try_from(x).map_err(|_| ElemError::SliceOverflow { offset: offset.clone(), length: length.clone() }))?;
    //     let u_offset_plus_length = u_offset.checked_add(u_length)
    //         .ok_or_else(|| ElemError::SliceOverflow { offset: offset.clone(), length: length.clone() })?;
    //     if iterable.clone().into_iter().count() < u_offset_plus_length {
    //         Err(ElemError::SliceTooShort {
    //             offset: u_offset,
    //             length: u_length,
    //             iterable: From::from(elem_symbol),
    //         })
    //     } else {
    //         Ok(iterable.into_iter().skip(u_offset).take(u_length).collect())
    //     }
    // }

    // pub fn slice(maybe_offset: Self, maybe_length: Self, maybe_iterable: Self) -> Result<Self, ElemError> {
    //     match (maybe_offset, maybe_length, maybe_iterable) {
    //         (Self::Number(offset), Self::Number(length), Self::Bytes(iterator)) =>
    //             Ok(Self::Bytes(Self::slice_generic(offset, length, iterator, ElemSymbol::Bytes)?)),
    //         (Self::Number(offset), Self::Number(length), Self::String(iterator)) => {
    //             let iterator_vec = Vec::from(iterator.clone());
    //             Ok(Self::String(String::from_utf8(Self::slice_generic(offset.clone(), length.clone(), iterator_vec, ElemSymbol::String)?)
    //                     .map_err(|_| ElemError::SliceInvalidUTF8 { offset: offset, length: length, iterator: iterator })?))
    //             },
    //         (Self::Number(offset), Self::Number(length), Self::Array(iterator)) =>
    //             Ok(Self::Array(Self::slice_generic(offset, length, iterator, ElemSymbol::Number)?)),
    //         (Self::Number(offset), Self::Number(length), Self::Object(iterator)) =>
    //             Ok(Self::Object(Self::slice_generic(offset, length, iterator, ElemSymbol::Object)?)),
    //         (maybe_not_offset, maybe_not_length, maybe_not_iterable) => {
    //             Err(ElemError::SliceUnsupportedTypes {
    //                 maybe_not_offset: maybe_not_offset.symbol_str(),
    //                 maybe_not_length: maybe_not_length.symbol_str(),
    //                 maybe_not_iterable: maybe_not_iterable.symbol_str(),
    //             })
    //         }
    //     }
    // }

    // fn index_generic<T: Clone + IntoIterator +
    //     std::iter::FromIterator<<T as std::iter::IntoIterator>::Item>>(index: Number,
    //                                                                    iterable: T,
    //                                                                    elem_symbol: ElemSymbol) ->
    //   Result<<T as std::iter::IntoIterator>::Item, ElemError> {
    //     let u_index: usize = index.as_u64()
    //         .ok_or_else(|| ElemError::IndexNotU64(index.clone()))
    //         .and_then(|x| usize::try_from(x).map_err(|_| ElemError::IndexOverflow(index.clone())))?;
    //     if iterable.clone().into_iter().count() <= u_index {
    //         return Err(ElemError::IndexTooShort {
    //             index: u_index,
    //             iterable: From::from(elem_symbol),
    //         })
    //     } else {
    //         match iterable.into_iter().skip(u_index).next() {
    //             None => Err(ElemError::IndexTooShort { index: u_index, iterable: From::from(elem_symbol) }),
    //             Some(x) => Ok(x),
    //         }
    //     }
    // }

    // pub fn index(self, maybe_iterable: Self) -> Result<Self, ElemError> {
    //     match (self, maybe_iterable) {
    //         // (Self::Number(index), Self::Bytes(iterator)) =>
    //         //     Ok(Self::Bytes(vec![Self::index_generic(index, iterator, ElemSymbol::Bytes)?])),
    //         (Self::Number(index), Self::Array(iterator)) =>
    //             Ok(Self::Json(Self::index_generic(index, iterator, ElemSymbol::Json)?)),
    //         (Self::Number(index), Self::Object(iterator)) =>
    //             Ok(Self::Json(Self::index_generic(index, iterator, ElemSymbol::Object)?.1)),
    //         (maybe_not_index, maybe_not_iterable) => {
    //             Err(ElemError::IndexUnsupportedTypes {
    //                 maybe_not_index: maybe_not_index.symbol_str(),
    //                 maybe_not_iterable: maybe_not_iterable.symbol_str(),
    //             })
    //         }
    //     }
    // }

    // pub fn to_json(self) -> Result<Self, ElemError> {
    //     Ok(Self::Json(serde_json::to_value(self)?))
    // }

    // pub fn unpack_json(self, elem_symbol: ElemSymbol) -> Result<Self, ElemError> {
    //     match (self, elem_symbol) {
    //         (Self::Json(serde_json::Value::Null), ElemSymbol::Unit) => Ok(Self::Unit),
    //         (Self::Json(serde_json::Value::Bool(x)), ElemSymbol::Bool) => Ok(Self::Bool(x)),
    //         (Self::Json(serde_json::Value::Number(x)), ElemSymbol::Number) => Ok(Self::Number(x)),
    //         (Self::Json(serde_json::Value::String(x)), ElemSymbol::String) => Ok(Self::String(x)),
    //         (Self::Json(serde_json::Value::Array(x)), ElemSymbol::Array) => Ok(Self::Array(x)),
    //         (Self::Json(serde_json::Value::Object(x)), ElemSymbol::Object) => Ok(Self::Object(x)),
    //         (Self::Json(json), elem_symbol) => Err(ElemError::UnpackJsonUnsupportedSymbol {
    //           json: json,
    //           elem_symbol: From::from(elem_symbol),
    //         }),
    //         (non_json, _) => Err(ElemError::UnpackJsonUnexpectedType {
    //               non_json: non_json.symbol_str(),
    //               elem_symbol: From::from(elem_symbol),
    //         }),
    //     }
    // }

    // pub fn string_to_bytes(self) -> Result<Self, ElemError> {
    //     match self {
    //         Self::String(x) => Ok(Self::Bytes(x.into_bytes())),
    //         other => Err(ElemError::StringToBytesUnsupportedType(other.symbol_str())),
    //     }
    // }






//     ToJson,
//     Restack(Restack),

// #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
// pub enum Instruction {
//     Push(Elem),
//     Restack(Restack),
//     HashSha256,
//     CheckLe,
//     CheckLt,
//     CheckEq,
//     Concat,
//     Slice,
//     Index,
//     Lookup,
//     AssertTrue,
//     ToJson,
//     UnpackJson(ElemSymbol),
//     StringToBytes,
// }



// pub fn demo_triple() -> ConsElem<(), TBool, ConsElem<(), TUnit, ConsElem<(), TBool, Nil<()>>>> {
//     Nil { t: PhantomData }
//     .cons(TBool { get_bool: true })
//     .cons(TUnit { })
//     .cons(TBool { get_bool: false })
// }

// pub fn demo_triple_with_tl_handles_intermediate_types() -> ConsElem<(), TBool, ConsElem<(), TUnit, ConsElem<(), TBool, Nil<()>>>> {
//     Nil { t: PhantomData }
//     .cons(TBool { get_bool: true })
//     .cons(TUnit { })
//     .cons(TBool { get_bool: false })
//     .cons(TBool { get_bool: true })
//     .cons(TUnit { })
//     .tl()
//     .tl()
// }




































//////
////////////
//////
////////////
//////////////////
////////////
//////
////////////
//////
////////////
//////////////////
////////////
//////////////////
////////////////////////
//////////////////
////////////
//////////////////
////////////
//////
////////////
//////
////////////
//////////////////
////////////
//////
////////////
//////
////////////
//////////////////
////////////
//////////////////
////////////////////////
//////////////////
////////////
//////////////////
////////////
//////////////////
////////////////////////
//////////////////
////////////////////////
//////////////////////////////
////////////////////////
//////////////////
////////////////////////
//////////////////
////////////
//////////////////
////////////
//////////////////
////////////////////////
//////////////////
////////////
//////////////////
////////////
//////
////////////
//////
////////////
//////////////////
////////////
//////
////////////
//////
////////////
//////////////////
////////////
//////////////////
////////////////////////
//////////////////
////////////
//////////////////
////////////
//////
////////////
//////
////////////
//////////////////
////////////
//////
////////////
//////


// typing:
// - unification
//   + inference
//   + checking against inferred or other type (this + inference = bidirecitonal)
// - two categories of tests:
//   + property tests for typing methods themselves
//   + test that a function having a particular type -> it runs w/o type errors on such inputs

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BaseElemType {
    Any,
    Concat,
    Index,
    Slice,
    ElemSymbol(ElemSymbol),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ElemTypeInfo {
    base_elem_type: BaseElemType,
    location: Location,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ElemType {
    type_set: EnumSet<ElemSymbol>,
    info: Vec<ElemTypeInfo>,
}

// Formatting:
// ```
// ElemType {
//     type_set: {A, B, C},
//     info: _,
// }
// ```
//
// Results in:
// ```
// {A, B, C}
// ```
impl Display for ElemType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f,
               "{{{}}}",
               self.type_set.iter()
               .fold(String::new(),
                     |memo, x| {
                         let x_str: &'static str = From::from(x);
                         if memo == "" {
                            x_str.to_string()
                         } else {
                            memo + ", " + &x_str.to_string()
                         }
                    }
               ))
    }
}

#[cfg(test)]
mod elem_type_display_tests {
    use super::*;

    #[test]
    fn test_empty() {
        let elem_type = ElemType {
            type_set: EnumSet::empty(),
            info: vec![],
        };
        assert_eq!("{}", format!("{}", elem_type));
    }

    #[test]
    fn test_singleton() {
        for elem_symbol in EnumSet::all().iter() {
            let elem_type = ElemType {
                type_set: EnumSet::only(elem_symbol),
                info: vec![],
            };
            assert_eq!(format!("{{{}}}", Into::<&'static str>::into(elem_symbol)),
                       format!("{}", elem_type));
        }
    }

    #[test]
    fn test_all() {
        assert_eq!("{Unit, Bool, Number, Bytes, String, Array, Object, JSON}",
                   format!("{}", ElemType::any(vec![])));
    }
}

impl ElemSymbol {
    pub fn elem_type(&self, locations: Vec<Location>) -> ElemType {
        ElemType {
            type_set: EnumSet::only(*self),
            info: locations.iter()
                .map(|&location|
                     ElemTypeInfo {
                         base_elem_type: BaseElemType::ElemSymbol(*self),
                         location: location,
                    }).collect(),
        }
    }
}

impl Elem {
    pub fn elem_type(&self, locations: Vec<Location>) -> ElemType {
        self.symbol().elem_type(locations)
    }
}

impl ElemType {
    fn from_locations(type_set: EnumSet<ElemSymbol>,
                      base_elem_type: BaseElemType,
                      locations: Vec<Location>) -> Self {
        ElemType {
            type_set: type_set,
            info: locations.iter()
                .map(|&location|
                     ElemTypeInfo {
                         base_elem_type: base_elem_type,
                         location: location,
                    }).collect(),
        }
    }

    pub fn any(locations: Vec<Location>) -> Self {
        Self::from_locations(
            EnumSet::all(),
            BaseElemType::Any,
            locations)
    }

    pub fn concat_type(locations: Vec<Location>) -> Self {
        Self::from_locations(
            enum_set!(ElemSymbol::Bytes |
                      ElemSymbol::String |
                      ElemSymbol::Array |
                      ElemSymbol::Object),
            BaseElemType::Concat,
            locations)
    }

    pub fn index_type(locations: Vec<Location>) -> Self {
        Self::from_locations(
            enum_set!(ElemSymbol::Array |
                      ElemSymbol::Object),
            BaseElemType::Index,
            locations)
    }

    pub fn slice_type(locations: Vec<Location>) -> Self {
        Self::concat_type(locations)
    }

    pub fn unify(&self, other: Self) -> Result<Self, ElemTypeError> {
        let both = self.type_set.intersection(other.type_set);
        if both.is_empty() {
            Err(ElemTypeError::UnifyEmpty {
                lhs: self.clone(),
                rhs: other.clone(),
            })
        } else {
            let mut both_info = self.info.clone();
            both_info.append(&mut other.info.clone());
            Ok(ElemType {
                type_set: both,
                info: both_info,
            })
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypeId {
    type_id: usize,
}

impl TypeId {
    // TODO: test by checking:
    // xs.map(TypeId).fold(x, offset) = TypeId(xs.fold(x, +))
    pub fn offset(&self, offset: TypeId) -> Self {
        TypeId {
            type_id: self.type_id + offset.type_id,
        }
    }

    pub fn update_type_id(&self, from: Self, to: Self) -> Self {
        if *self == from {
            to
        } else {
            *self
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Context {
    context: BTreeMap<TypeId, ElemType>,
    next_type_id: TypeId,
}


// Formatting:
// ```
// Context {
//     context: [
//         (t0, {A, B, C}),
//         (t1, {B, C}),
//         ..
//         (tN, {D, E, F})],
//     next_type_id: N+1,
// }
// ```
//
// Results in:
// ```
// ∀ (t0 ∊ {A, B, C}),
// ∀ (t1 ∊ {B, C}),
// ..
// ∀ (tN ∊ {D, E, F}),
// ```
impl Display for Context {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
       write!(f,
              "{}",
              self.context.iter()
              .fold(String::new(), |memo, (i, xs)| {
                memo +
                "\n" +
                &format!("∀ (t{i} ∊ {xs}),", i = i.type_id, xs = xs).to_string()
              }))
    }
}

#[cfg(test)]
mod context_display_tests {
    use super::*;

    #[test]
    fn test_empty() {
        let big_type_id = TypeId {
            type_id: 2^32
        };
        let context = Context {
            context: BTreeMap::new(),
            next_type_id: big_type_id,
        };
        assert_eq!("", format!("{}", context));
    }

    #[test]
    fn test_singleton() {
        for elem_symbol in EnumSet::all().iter() {
            let elem_type = ElemType {
                type_set: EnumSet::only(elem_symbol),
                info: vec![],
            };
            let mut context_map = BTreeMap::new();
            context_map.insert(TypeId { type_id: 0 }, elem_type.clone());
            let context = Context {
                context: context_map,
                next_type_id: TypeId {
                    type_id: 1,
                },
            };
            assert_eq!(format!("\n∀ (t0 ∊ {}),", elem_type), format!("{}", context));
        }
    }
}

impl Context {
    pub fn new() -> Self {
        Context {
            context: BTreeMap::new(),
            next_type_id: TypeId {
                type_id: 0,
            },
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.context.keys().any(|x| *x >= self.next_type_id)
    }

    pub fn size(&self) -> usize {
        self.context.len()
    }

    pub fn push(&mut self, elem_type: ElemType) -> TypeId {
        let push_id = self.next_type_id;
        self.context.insert(push_id, elem_type);
        self.next_type_id = TypeId {
            type_id: push_id.type_id + 1,
        };
        push_id
    }

    // NormalizeOnInvalidBasis is possible iff a `TypeId` in (basis) is repeated
    // or missing from (self)
    pub fn normalize_on(&self, basis: Vec<TypeId>) -> Result<(Self, TypeIdMap), ContextError> {
        let mut source = self.clone();
        let mut result = Self::new();
        let mut type_map = TypeIdMap::new();
        for &type_id in &basis {
            match source.context.remove(&type_id) {
                None => Err(ContextError::NormalizeOnInvalidBasis {
                    type_id: type_id,
                    context: self.clone(),
                    basis: basis.clone().into_iter().collect(),
                }),
                Some(elem_type) => {
                    let new_type_id = result.next_type_id;
                    result.push(elem_type);
                    type_map.push(type_id, new_type_id)?;
                    Ok(())
                },
            }?
        }
        Ok((result, type_map))
    }

    pub fn offset(&self, offset: TypeId) -> Self {
        Context {
            context: self.context.iter().map(|(k, x)| (k.offset(offset), x.clone())).collect(),
            next_type_id: self.next_type_id.offset(offset),
        }
    }

    pub fn update_type_id(&mut self, from: TypeId, to: TypeId) -> Result<(), ContextError> {
        if self.context.contains_key(&from) {
            Ok(())
        } else {
            Err(ContextError::UpdateTypeIdFromMissing {
                from: from,
                to: to,
                context: self.clone(),
            })
        }?;
        if self.context.contains_key(&to) {
            Err(ContextError::UpdateTypeIdToPresent {
                from: from,
                to: to,
                context: self.clone(),
            })
        } else {
            Ok(())
        }?;
        self.context = self.context.iter().map(|(k, x)| (k.update_type_id(from, to), x.clone())).collect();
        self.next_type_id = cmp::max(self.next_type_id, to);
        Ok(())
    }

    pub fn disjoint_union(&mut self, other: Self) -> Result<(), ContextError> {
        for (&type_id, elem_type) in other.context.iter() {
            match self.context.insert(type_id, elem_type.clone()) {
                None => {
                    Ok(())
                },
                Some(conflicting_elem_type) => Err(ContextError::DisjointUnion {
                    type_id: type_id,
                    elem_type: elem_type.clone(),
                    conflicting_elem_type: conflicting_elem_type,
                    lhs: self.clone(),
                    rhs: other.clone(),
                }),
            }?
        }
        self.next_type_id = cmp::max(self.next_type_id, other.next_type_id);
        Ok(())
    }

    pub fn get(&mut self, index: &TypeId, error: &dyn Fn() -> ContextError) -> Result<ElemType, ContextError> {
        Ok(self.context.get(index).ok_or_else(|| ContextError::GetUnknownTypeId {
            context: self.clone(),
            index: *index,
            error: Box::new(error()),
        })?.clone())
    }

    // unify the types of two TypeId's into the rhs
    // removing the lhs
    pub fn unify(&mut self, xi: TypeId, yi: TypeId) -> Result<(), ContextError> {
        let x_type = self.context.remove(&xi).ok_or_else(|| ContextError::Unify {
            xs: self.clone(),
            xi: xi.clone(),
            yi: yi.clone(),
            is_lhs: true,
        })?;
        let y_type = self.context.remove(&yi).ok_or_else(|| ContextError::Unify {
            xs: self.clone(),
            xi: xi.clone(),
            yi: yi.clone(),
            is_lhs: false,
        })?;
        let xy_type = x_type.unify(y_type).or_else(|e| Err(ContextError::UnifyElemType {
            xs: self.clone(),
            xi: xi.clone(),
            yi: yi.clone(),
            error: e,
        }))?;
        self.context.insert(yi, xy_type);
        Ok(())
    }
}


#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Type {
    pub context: Context,
    pub i_type: Vec<TypeId>,
    pub o_type: Vec<TypeId>,
}

impl Type {
    pub fn id() -> Self {
        Type {
            context: Context::new(),
            i_type: vec![],
            o_type: vec![],
        }
    }

    pub fn next_type_id(&self) -> TypeId {
        self.context.next_type_id
    }

    // check whether all the TypeId's are valid
    pub fn is_valid(&self) -> bool {
        let next_type_id = self.next_type_id();
        self.context.is_valid() &&
        !(self.i_type.iter().any(|x| *x >= next_type_id) ||
          self.o_type.iter().any(|x| *x >= next_type_id))
    }

    // equivalent to running update_type_id w/ offset from largest to smallest
    // existing TypeId
    pub fn offset(&self, offset: TypeId) -> Self {
        Type {
            context: self.context.offset(offset),
            i_type: self.i_type.iter().map(|x| x.offset(offset)).collect(),
            o_type: self.o_type.iter().map(|x| x.offset(offset)).collect(),
        }
    }

    pub fn update_type_id(&mut self, from: TypeId, to: TypeId) -> Result<(), TypeError> {
        self.context.update_type_id(from, to).map_err(|e| TypeError::UpdateTypeId(e))?;
        self.i_type = self.i_type.iter().map(|x| x.update_type_id(from, to)).collect();
        self.o_type = self.o_type.iter().map(|x| x.update_type_id(from, to)).collect();
        Ok(())
    }

    pub fn normalize(&self) -> Result<Self, TypeError> {
        let mut basis = self.i_type.clone();
        basis.append(&mut self.o_type.clone());
        basis.dedup();
        let (new_context, type_map) = self.context.normalize_on(basis).map_err(|e| TypeError::ContextError(e))?;
        Ok(Type {
            context: new_context,
            i_type: type_map.run(self.i_type.clone()).map_err(|e| TypeError::TypeIdMapError(e))?,
            o_type: type_map.run(self.o_type.clone()).map_err(|e| TypeError::TypeIdMapError(e))?,
        })
    }

    // f : self
    // g : other
    // self.compose(other) : (f ++ g).type_of()
    //
    // input ->
    // other.i_type
    // other.o_type
    // self.i_type
    // self.o_type
    // -> output
    //
    // 1. iterate through (zip(self.o_type, other.i_type)) and unify the pairs into a new context
    // 2. collect the remainder and add them to the context
    // 3. add the remainder to (self.i_type, other.o_type), with replaced variables
    pub fn compose(&self, other: Self) -> Result<Self, TypeError> {
        println!("");
        println!("composing:\n{0}\n\nAND\n{1}\n", self, other);

        let mut context = self.context.clone();
        // println!("context: {}", context);
        // println!("context.next_type_id: {:?}", context.next_type_id.type_id);

        let offset_other = other.offset(self.next_type_id());
        // println!("offset_other: {}", offset_other);

        context.disjoint_union(offset_other.context.clone())
            .map_err(|e| TypeError::ContextError(e))?;
        // println!("context union: {}", context);

        let mut mut_offset_other = offset_other.clone();
        let mut zip_len = 0;
        let other_o_type = offset_other.o_type.iter().clone();
        let self_i_type = self.i_type.iter().clone();
        other_o_type.zip(self_i_type).try_for_each(|(&o_type, &i_type)| {
            zip_len += 1;
            context
                .unify(o_type, i_type)
                .map_err(|e| TypeError::ContextError(e))?;
            mut_offset_other
                .update_type_id(o_type, i_type)?;
            Ok(())
        })?;

        Ok(Type {
            context: context,
            i_type: mut_offset_other.i_type.iter().chain(self.i_type.iter().skip(zip_len)).copied().collect(),
            o_type: self.o_type.iter().chain(mut_offset_other.o_type.iter().skip(zip_len)).copied().collect(),
        })
    }
}

// Formatting:
// ```
// Type {
//     context: Context {
//         context: [
//             (t0, {A, B, C}),
//             (t1, {B, C}),
//             ..
//             (tN, {D, E, F})],
//         next_type_id: N+1,
//     },
//     i_type: [0, 1, .., N],
//     0_type: [i, j, .., k],
// }
// ```
//
// Results in:
// ```
// ∀ (t0 ∊ {A, B, C}),
// ∀ (t1 ∊ {B, C}),
// ..
// ∀ (tN ∊ {D, E, F}),
// [t0, t1, .., tN] ->
// [ti, tj, .., tk]
// ```
impl Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        // let self_normalized = self.normalize().map_err(|_| fmt::Error)?;
        let self_normalized = self;
        write!(f,
               "{context}\n[{i_type}] ->\n[{o_type}]",
               context = self_normalized.context,
               i_type = self_normalized.i_type.iter().fold(String::new(), |memo, x| {
                   let x_str = format!("t{}", x.type_id);
                   if memo == "" {
                       x_str
                   } else {
                       memo + ", " + &x_str.to_string()
                   }}),
               o_type = self_normalized.o_type.iter().fold(String::new(), |memo, x| {
                   let x_str = format!("t{}", x.type_id);
                   if memo == "" {
                       x_str
                   } else {
                       memo + ", " + &x_str.to_string()
                   }}))
    }
}

#[cfg(test)]
mod type_display_tests {
    use super::*;

    #[test]
    fn test_empty() {
        let big_type_id = TypeId {
            type_id: 2^32
        };
        let context = Context {
            context: BTreeMap::new(),
            next_type_id: big_type_id,
        };
        let example_type = Type {
            context: context,
            i_type: vec![],
            o_type: vec![],
        };
        assert_eq!("\n[] ->\n[]", format!("{}", example_type));
    }

    #[test]
    fn test_singleton() {
        for elem_symbol in EnumSet::all().iter() {
            let elem_type = ElemType {
                type_set: EnumSet::only(elem_symbol),
                info: vec![],
            };
            let mut context_map = BTreeMap::new();
            context_map.insert(TypeId { type_id: 0 }, elem_type.clone());
            let context = Context {
                context: context_map,
                next_type_id: TypeId {
                    type_id: 1,
                },
            };
            let example_type = Type {
                context: context,
                i_type: vec![TypeId { type_id: 0 }, TypeId { type_id: 0 }],
                o_type: vec![TypeId { type_id: 0 }],
            };
            assert_eq!(format!("\n∀ (t0 ∊ {}),\n[t0, t0] ->\n[t0]", elem_type), format!("{}", example_type));
        }
    }
}



// TODO: split up TypeError
// TODO: add layers of detail to TypeIdMapGetUnknownTypeId


#[derive(Debug, PartialEq, Error)]
pub enum ElemTypeError {
    #[error("ElemType::unify applied to non-intersecting types: lhs: {lhs:?}; rhs: {rhs:?}")]
    UnifyEmpty {
        lhs: ElemType,
        rhs: ElemType,
        // location: TyUnifyLocation,
    },
}

#[derive(Debug, PartialEq, Error)]
pub enum TypeIdMapError {
    #[error("TypeIdMap::get attempted to get a TypeId: {index:?}, not in the map: {type_map:?}; at location in TypeIdMap::run {location:?}")]
    GetUnknownTypeId {
        index: TypeId,
        location: usize,
        type_map: TypeIdMap,
    },

    #[error("TypeIdMap::push already exists: mapping from: {from:?}, to: {to:?}, in TypeIdMap {map:?}")]
    PushExists {
        from: TypeId,
        to: TypeId,
        map: TypeIdMap,
    },
}

#[derive(Debug, PartialEq, Error)]
pub enum ContextError {
    #[error("Context::get applied to a TypeId: {index:?}, not in the Context: {context:?}, error: {error:?}")]
    GetUnknownTypeId {
        context: Context,
        index: TypeId,
        error: Box<Self>,
    },

    #[error("Context::disjoint_union applied to lhs: {lhs:?}, and rhs: {rhs:?}, /
            with type_id: {type_id:?}, and elem_type: {elem_type:?}, conflicted /
            with lhs entry conflicting_elem_type: {conflicting_elem_type:?}")]
    DisjointUnion {
        type_id: TypeId,
        elem_type: ElemType,
        conflicting_elem_type: ElemType,
        lhs: Context,
        rhs: Context,
    },

    #[error("Context::normalize_on applied to invalid basis: type_id: {type_id:?}, context: {context:?}, basis: {basis:?}")]
    NormalizeOnInvalidBasis {
        type_id: TypeId,
        context: Context,
        basis: Vec<TypeId>,
    },

    #[error("Context::update_type_id called on missing 'from: TypeId':\n from: {from:?}\n to: {to:?}\n context: {context:?}")]
    UpdateTypeIdFromMissing {
        from: TypeId,
        to: TypeId,
        context: Context,
    },

    #[error("Context::update_type_id called on already-present 'to: TypeId':\n from: {from:?}\n to: {to:?}\n context: {context:?}")]
    UpdateTypeIdToPresent {
        from: TypeId,
        to: TypeId,
        context: Context,
    },

    #[error("Context::unify failed:\n xs: {xs:?}\n xi: {xi:?}\n yi: {yi:?}\n is_lhs: {is_lhs:?}\n")]
    Unify {
            xs: Context,
            xi: TypeId,
            yi: TypeId,
            is_lhs: bool,
    },

    #[error("Context::unify failed to unify ElemType's:\n xs: {xs:?}\n xi: {xi:?}\n yi: {yi:?}\n elem_error: {error:?}\n")]
    UnifyElemType {
            xs: Context,
            xi: TypeId,
            yi: TypeId,
            error: ElemTypeError,
    },

    #[error("Context::normalize_on building TypeIdMap failed: {0:?}")]
    TypeIdMapError(TypeIdMapError),
}

impl From<TypeIdMapError> for ContextError {
    fn from(error: TypeIdMapError) -> Self {
        Self::TypeIdMapError(error)
    }
}


#[derive(Debug, PartialEq, Error)]
pub enum TypeError {
    #[error("ContextError {0}")]
    ContextError(ContextError),

    #[error("TypeError::update_type_id failed when updating the Context: {0}")]
    UpdateTypeId(ContextError),

    #[error("TypeError::compose disjoint_union {0}")]
    ComposeDisjointUnion(ContextError),

    #[error("Type::normalize applying TypeIdMap failed: {0:?}")]
    TypeIdMapError(TypeIdMapError),
}


#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypeIdMap {
    map: BTreeMap<TypeId, TypeId>,
}


impl TypeIdMap {
    pub fn new() -> Self {
        TypeIdMap {
            map: BTreeMap::new(),
        }
    }

    pub fn push(&mut self, from: TypeId, to: TypeId) -> Result<(), TypeIdMapError> {
        if self.map.contains_key(&from) {
            Err(TypeIdMapError::PushExists {
                from: from,
                to: to,
                map: self.clone(),
            })
        } else {
            self.map.insert(from, to);
            Ok(())
        }
    }

    pub fn get(&self, index: &TypeId, location: usize) -> Result<&TypeId, TypeIdMapError> {
        self.map.get(index)
            .ok_or_else(|| TypeIdMapError::GetUnknownTypeId {
                index: index.clone(),
                location: location,
                type_map: self.clone(),
            })
    }

    pub fn run(&self, type_vars: Vec<TypeId>) -> Result<Vec<TypeId>, TypeIdMapError> {
        type_vars.iter().enumerate().map(|(i, x)| Ok(self.get(x, i)?.clone())).collect()
    }
}

