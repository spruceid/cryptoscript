use crate::restack::Restack;

use thiserror::Error;

// use core::num;
use std::cmp;
use std::convert::TryFrom;
// use std::iter::Chain;
// use std::iter::Repeat;
// use std::iter::Zip;
// use std::iter::empty;
use std::ops::Range;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Number, Value};

use enumset::{EnumSet, EnumSetType};

use quickcheck::{empty_shrinker, Arbitrary, Gen};

// - readability:

//     + stack size typing
//     + stack typing
//     + value annotations
//     + JSON traversal path

// TODO - primitives + numeric: to_int, add, sub, mul, div
// TODO - primitives + bool: neg, and, or
// TODO - primitives + signatures: to_pub_key, check_sig

// - debugging

// TODO: step through / display steps better

// - testing

// TODO: property based tests


#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Elem {
    Unit,
    Bool(bool),
    Number(Number),
    Bytes(Vec<u8>),
    String(String),
    Array(Vec<Value>),
    Object(Map<String, Value>),
    Json(Value),
}

impl PartialOrd for Elem {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match (self, other) {
            (Self::Unit, Self::Unit) => Some(cmp::Ordering::Equal),
            (Self::Bool(x), Self::Bool(y)) => x.partial_cmp(y),
            (Self::Bytes(x), Self::Bytes(y)) => x.partial_cmp(y),
            (Self::Number(x), Self::Number(y)) => format!("{}", x).partial_cmp(&format!("{}", y)),
            // TODO: use x.to_string().partial_cmp(&y.to_string()),

            (Self::String(x), Self::String(y)) => x.partial_cmp(y),
            (Self::Array(x), Self::Array(y)) => if x == y { Some(cmp::Ordering::Equal) } else { None },
            (Self::Object(x), Self::Object(y)) => if x == y { Some(cmp::Ordering::Equal) } else { None }
            (_, _) => None,
        }
    }
}


// EnumSetType implies: Copy, PartialEq, Eq
#[derive(EnumSetType, Debug, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ElemSymbol {
    Unit,
    Bool,
    Number,
    Bytes,
    String,
    Array,
    Object,
    Json,
}

impl Arbitrary for ElemSymbol {
    fn arbitrary(g: &mut Gen) -> Self {
        let choices: Vec<ElemSymbol> = EnumSet::all().iter().collect();
        *g.choose(&choices).unwrap_or_else(|| &Self::Unit)
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let self_copy = self.clone();
        Box::new(EnumSet::all().iter().filter(move |&x| x < self_copy))
    }
}




#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArbitraryNumber {
    number: Number,
}

impl Arbitrary for ArbitraryNumber {
    fn arbitrary(g: &mut Gen) -> Self {
        if Arbitrary::arbitrary(g) {
            if Arbitrary::arbitrary(g) {
                let x: u64 = Arbitrary::arbitrary(g);
                ArbitraryNumber { number:
                    From::from(x)
                }
            } else {
                let x: i64 = Arbitrary::arbitrary(g);
                ArbitraryNumber { number:
                    From::from(x)
                }
            }
        } else {
            let x: f64 = Arbitrary::arbitrary(g);
            ArbitraryNumber { number:
                Number::from_f64(x).unwrap_or(From::from(0u8))
            }
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        match self.number.as_f64() {
            None => match self.number.as_u64() {
                None => match self.number.as_i64() {
                    None => empty_shrinker(),
                    Some(self_i64) => Box::new(
                        self_i64.shrink()
                        .map(|x| ArbitraryNumber {
                            number: From::from(x),
                        })),
                },
                Some(self_u64) => Box::new(
                    self_u64.shrink()
                    .map(|x| ArbitraryNumber {
                        number: From::from(x),
                    })),
            },
            Some(self_f64) => Box::new(
                self_f64.shrink()
                .map(|x| ArbitraryNumber {
                    number: Number::from_f64(x).unwrap_or(From::from(0u8)),
                })),
        }
    }
}

impl ElemSymbol {
    pub fn arbitrary_contents(&self, g: &mut Gen) -> Elem {
        match self {
            Self::Unit => Elem::Unit,
            Self::Bool => Elem::Bool(Arbitrary::arbitrary(g)),
            Self::Number => {
                let x: ArbitraryNumber = Arbitrary::arbitrary(g);
                Elem::Number(x.number)
            },
            Self::Bytes => Elem::Bytes(Arbitrary::arbitrary(g)),
            Self::String => Elem::String(Arbitrary::arbitrary(g)),
            Self::Array => Elem::Array(Arbitrary::arbitrary(g)),
            Self::Object => Elem::Object(Arbitrary::arbitrary(g)),
            Self::Json => Elem::Json(Arbitrary::arbitrary(g)),
        }
    }
}


impl From<ElemSymbol> for &'static str {
    fn from(x: ElemSymbol) -> Self {
        match x {
            ElemSymbol::Unit => "Unit",
            ElemSymbol::Bool => "Bool",
            ElemSymbol::Bytes => "Bytes",
            ElemSymbol::Number => "Number",
            ElemSymbol::String => "String",
            ElemSymbol::Array => "Array",
            ElemSymbol::Object => "Object",
            ElemSymbol::Json => "JSON",
        }
    }
}

impl From<&Elem> for ElemSymbol {
    fn from(x: &Elem) -> Self {
        match x {
            Elem::Unit => Self::Unit,
            Elem::Bool(_) => Self::Bool,
            Elem::Number(_) => Self::Number,
            Elem::Bytes(_) => Self::Bytes,
            Elem::String(_) => Self::String,
            Elem::Array(_) => Self::Array,
            Elem::Object(_) => Self::Object,
            Elem::Json(_) => Self::Json,
        }
    }
}

impl ElemSymbol {
    #[cfg(test)]
    pub fn default_elem(&self) -> Elem {
        match self {
            Self::Unit => Elem::Unit,
            Self::Bool => Elem::Bool(Default::default()),
            Self::Number => Elem::Number(Default::default()),
            Self::Bytes => Elem::Bytes(Default::default()),
            Self::String => Elem::String(Default::default()),
            Self::Array => Elem::Array(Default::default()),
            Self::Object => Elem::Object(Default::default()),
            Self::Json => Elem::Json(Default::default()),
        }
    }

    pub fn ty(&self) -> Ty {
        Ty {
            ty_set: EnumSet::only(self.clone()),
        }
    }
}

#[cfg(test)]
mod elem_symbol_tests {
    use super::*;

    #[test]
    fn test_from_default_elem() {
        for symbol in EnumSet::all().iter() {
            assert_eq!(symbol, From::from(symbol.default_elem()))
        }
    }

    #[test]
    fn test_to_default_elem() {
        for default_elem in [
          Elem::Unit,
          Elem::Bool(Default::default()),
          Elem::Number(Default::default()),
          Elem::Bytes(Default::default()),
          Elem::String(Default::default()),
          Elem::Array(Default::default()),
          Elem::Object(Default::default()),
          Elem::Json(Default::default()),
        ] {
            assert_eq!(default_elem, From::from(default_elem).default_elem())
        }
    }
}

impl Elem {
    pub fn symbol(&self) -> ElemSymbol {
      From::from(self)
    }

    pub fn symbol_str(&self) -> &'static str {
      From::from(self.symbol())
    }

    // pub fn push_ty_sets(&self) -> (Vec<Ty>, Vec<Ty>) {
    //     (vec![], vec![self.symbol().ty()])
    // }

    pub fn assert_true(&self) -> Result<(), ElemError> {
        match self {
            Self::Bool(x) => if *x {
                    Ok(())
                } else {
                    Err(ElemError::AssertTrueFailed())
                },
            found => Err(ElemError::AssertTrueUnsupportedType(found.clone())),
        }
    }

    pub fn check_le(self, other: Self) -> Result<Self, ElemError> {
        let result = match self.partial_cmp(&other)
            .ok_or_else(|| ElemError::CheckLeIncomparableTypes {
                lhs: self.symbol_str(),
                rhs: other.symbol_str() })? {
                    cmp::Ordering::Less => true,
                    cmp::Ordering::Equal => true,
                    cmp::Ordering::Greater => false,
        };
        Ok(Self::Bool(result))
    }

    pub fn check_lt(self, other: Self) -> Result<Self, ElemError> {
        let result = match self.partial_cmp(&other)
            .ok_or_else(|| ElemError::CheckLtIncomparableTypes {
                lhs: self.symbol_str(),
                rhs: other.symbol_str() })? {
                    cmp::Ordering::Less => true,
                    _ => false,
        };
        Ok(Self::Bool(result))
    }

    pub fn check_eq(self, other: Self) -> Result<Self, ElemError> {
        let result = match self.partial_cmp(&other)
            .ok_or_else(|| ElemError::CheckEqIncomparableTypes {
                lhs: self.symbol_str(),
                rhs: other.symbol_str() })? {
                    cmp::Ordering::Equal => true,
                    _ => false,
        };
        Ok(Self::Bool(result))
    }

    fn concat_generic<T: IntoIterator + std::iter::FromIterator<<T as std::iter::IntoIterator>::Item>>(x: T, y: T) -> T {
        x.into_iter().chain(y.into_iter()).collect()
    }

    pub fn concat(self, other: Self) -> Result<Self, ElemError> {
        match (self, other) {
            (Self::Bytes(x), Self::Bytes(y)) => Ok(Self::Bytes(Self::concat_generic(x, y))),
            (Self::String(x), Self::String(y)) => {
                Ok(Self::String(String::from_utf8(Self::concat_generic(Vec::from(x.clone()), Vec::from(y.clone())))
                                .map_err(|_| ElemError::ConcatInvalidUTF8 { lhs: x, rhs: y })?))
            },
            (Self::Array(x), Self::Array(y)) => Ok(Self::Array(Self::concat_generic(x, y))),
            (Self::Object(x), Self::Object(y)) => Ok(Self::Object(Self::concat_generic(x, y))),
            (some_x, some_y) => {
                Err(ElemError::ConcatUnsupportedTypes {
                    lhs: some_x.symbol_str(),
                    rhs: some_y.symbol_str()
                })
            },
        }
    }

    fn slice_generic<T: Clone + IntoIterator +
      std::iter::FromIterator<<T as std::iter::IntoIterator>::Item>>(offset: Number,
                                                                     length: Number,
                                                                     iterable: T,
                                                                     elem_symbol: ElemSymbol) ->
        Result<T, ElemError> {
        let u_offset = offset.as_u64()
            .ok_or_else(|| ElemError::SliceOffsetNotU64(offset.clone()))
            .and_then(|x| usize::try_from(x).map_err(|_| ElemError::SliceOverflow { offset: offset.clone(), length: length.clone() }))?;
        let u_length = length.as_u64()
            .ok_or_else(|| ElemError::SliceLengthNotU64(length.clone()))
            .and_then(|x| usize::try_from(x).map_err(|_| ElemError::SliceOverflow { offset: offset.clone(), length: length.clone() }))?;
        let u_offset_plus_length = u_offset.checked_add(u_length)
            .ok_or_else(|| ElemError::SliceOverflow { offset: offset.clone(), length: length.clone() })?;
        if iterable.clone().into_iter().count() < u_offset_plus_length {
            Err(ElemError::SliceTooShort {
                offset: u_offset,
                length: u_length,
                iterable: From::from(elem_symbol),
            })
        } else {
            Ok(iterable.into_iter().skip(u_offset).take(u_length).collect())
        }
    }

    pub fn slice(maybe_offset: Self, maybe_length: Self, maybe_iterable: Self) -> Result<Self, ElemError> {
        match (maybe_offset, maybe_length, maybe_iterable) {
            (Self::Number(offset), Self::Number(length), Self::Bytes(iterator)) =>
                Ok(Self::Bytes(Self::slice_generic(offset, length, iterator, ElemSymbol::Bytes)?)),
            (Self::Number(offset), Self::Number(length), Self::String(iterator)) => {
                let iterator_vec = Vec::from(iterator.clone());
                Ok(Self::String(String::from_utf8(Self::slice_generic(offset.clone(), length.clone(), iterator_vec, ElemSymbol::String)?)
                        .map_err(|_| ElemError::SliceInvalidUTF8 { offset: offset, length: length, iterator: iterator })?))
                },
            (Self::Number(offset), Self::Number(length), Self::Array(iterator)) =>
                Ok(Self::Array(Self::slice_generic(offset, length, iterator, ElemSymbol::Number)?)),
            (Self::Number(offset), Self::Number(length), Self::Object(iterator)) =>
                Ok(Self::Object(Self::slice_generic(offset, length, iterator, ElemSymbol::Object)?)),
            (maybe_not_offset, maybe_not_length, maybe_not_iterable) => {
                Err(ElemError::SliceUnsupportedTypes {
                    maybe_not_offset: maybe_not_offset.symbol_str(),
                    maybe_not_length: maybe_not_length.symbol_str(),
                    maybe_not_iterable: maybe_not_iterable.symbol_str(),
                })
            }
        }
    }

    fn index_generic<T: Clone + IntoIterator +
        std::iter::FromIterator<<T as std::iter::IntoIterator>::Item>>(index: Number,
                                                                       iterable: T,
                                                                       elem_symbol: ElemSymbol) ->
      Result<<T as std::iter::IntoIterator>::Item, ElemError> {
        let u_index: usize = index.as_u64()
            .ok_or_else(|| ElemError::IndexNotU64(index.clone()))
            .and_then(|x| usize::try_from(x).map_err(|_| ElemError::IndexOverflow(index.clone())))?;
        if iterable.clone().into_iter().count() <= u_index {
            return Err(ElemError::IndexTooShort {
                index: u_index,
                iterable: From::from(elem_symbol),
            })
        } else {
            match iterable.into_iter().skip(u_index).next() {
                None => Err(ElemError::IndexTooShort { index: u_index, iterable: From::from(elem_symbol) }),
                Some(x) => Ok(x),
            }
        }
    }

    pub fn index(self, maybe_iterable: Self) -> Result<Self, ElemError> {
        match (self, maybe_iterable) {
            // (Self::Number(index), Self::Bytes(iterator)) =>
            //     Ok(Self::Bytes(vec![Self::index_generic(index, iterator, ElemSymbol::Bytes)?])),
            (Self::Number(index), Self::Array(iterator)) =>
                Ok(Self::Json(Self::index_generic(index, iterator, ElemSymbol::Json)?)),
            (Self::Number(index), Self::Object(iterator)) =>
                Ok(Self::Json(Self::index_generic(index, iterator, ElemSymbol::Object)?.1)),
            (maybe_not_index, maybe_not_iterable) => {
                Err(ElemError::IndexUnsupportedTypes {
                    maybe_not_index: maybe_not_index.symbol_str(),
                    maybe_not_iterable: maybe_not_iterable.symbol_str(),
                })
            }
        }
    }

    // you can lookup a key in a Map (or fail, no recovery)
    pub fn lookup(self, maybe_map: Self) -> Result<Self, ElemError> {
        match (self, maybe_map) {
            (Self::String(key), Self::Object(map)) => {
                Ok(Self::Json(map.get(&key)
                    .ok_or_else(|| ElemError::LookupKeyMissing {
                        key: key,
                        map: map.clone(),
                    })
                    .map(|x|x.clone())?))
            },
            (maybe_not_key, maybe_not_map) => Err(ElemError::LookupUnsupportedTypes {
                maybe_not_key: maybe_not_key.symbol_str(),
                maybe_not_map: maybe_not_map.symbol_str(),
            }),
        }
    }

    pub fn sha256(self) -> Result<Self, ElemError> {
        match self {
            Self::Bytes(bytes) => {
                Ok(Self::Bytes(super::sha256(&bytes)))
            }
            elem => Err(ElemError::HashUnsupportedType(elem.symbol_str())),
        }
    }

    pub fn to_json(self) -> Result<Self, ElemError> {
        Ok(Self::Json(serde_json::to_value(self)?))
    }

    pub fn unpack_json(self, elem_symbol: ElemSymbol) -> Result<Self, ElemError> {
        match (self, elem_symbol) {
            (Self::Json(serde_json::Value::Null), ElemSymbol::Unit) => Ok(Self::Unit),
            (Self::Json(serde_json::Value::Bool(x)), ElemSymbol::Bool) => Ok(Self::Bool(x)),
            (Self::Json(serde_json::Value::Number(x)), ElemSymbol::Number) => Ok(Self::Number(x)),
            (Self::Json(serde_json::Value::String(x)), ElemSymbol::String) => Ok(Self::String(x)),
            (Self::Json(serde_json::Value::Array(x)), ElemSymbol::Array) => Ok(Self::Array(x)),
            (Self::Json(serde_json::Value::Object(x)), ElemSymbol::Object) => Ok(Self::Object(x)),
            (Self::Json(json), elem_symbol) => Err(ElemError::UnpackJsonUnsupportedSymbol {
              json: json,
              elem_symbol: From::from(elem_symbol),
            }),
            (non_json, _) => Err(ElemError::UnpackJsonUnexpectedType {
                  non_json: non_json.symbol_str(),
                  elem_symbol: From::from(elem_symbol),
            }),
        }
    }

    pub fn string_to_bytes(self) -> Result<Self, ElemError> {
        match self {
            Self::String(x) => Ok(Self::Bytes(x.into_bytes())),
            other => Err(ElemError::StringToBytesUnsupportedType(other.symbol_str())),
        }
    }
}

#[derive(Debug, PartialEq, Error)]
pub enum ElemError {
    #[error("expected Elem::Bool(true), found {0:?}")]
    AssertTrueUnsupportedType(Elem),
    #[error("expected true, but found false")]
    AssertTrueFailed(),
    #[error("check_le: incomparable types: {lhs:?}; {rhs:?}")]
    CheckLeIncomparableTypes {
        lhs: &'static str,
        rhs: &'static str,
    },
    #[error("check_lt: incomparable types: {lhs:?}; {rhs:?}")]
    CheckLtIncomparableTypes {
        lhs: &'static str,
        rhs: &'static str,
    },
    #[error("check_eq: incomparable types: {lhs:?}; {rhs:?}")]
    CheckEqIncomparableTypes {
        lhs: &'static str,
        rhs: &'static str,
    },
    #[error("concat applied to unsupported types: lhs: {lhs:?}; rhs: {rhs:?}")]
    ConcatUnsupportedTypes {
        lhs: &'static str,
        rhs: &'static str,
    },
    #[error("concat applied to strings that concatentate to invalid UTF8: lhs: {lhs:?}; rhs: {rhs:?}")]
    ConcatInvalidUTF8 {
        lhs: String,
        rhs: String,
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
    #[error("slice applied to arguments that produce invalid UTF8: offset: {offset:?}; length: {length:?}, iterator: {iterator:?}")]
    SliceInvalidUTF8 {
        offset: Number,
        length: Number,
        iterator: String,
    },

    #[error("index applied to unsupported types: maybe_not_index: {maybe_not_index:?}; maybe_not_iterable: {maybe_not_iterable:?}")]
    IndexUnsupportedTypes {
        maybe_not_index: &'static str,
        maybe_not_iterable: &'static str,
    },
    #[error("index applied to an 'index' that can't be unpacked to u64: {0:?}")]
    IndexNotU64(Number),
    #[error("index applied to an iterable that's too short for the given index: {index:?}; iterable: {iterable:?}")]
    IndexTooShort {
        index: usize,
        iterable: &'static str,
    },
    #[error("slice applied to offset and length whose sum overflows usize: {0:?}")]
    IndexOverflow(Number),

    #[error("lookup applied to unsupported types: maybe_not_key: {maybe_not_key:?}; maybe_not_map: {maybe_not_map:?}")]
    LookupUnsupportedTypes {
        maybe_not_key: &'static str,
        maybe_not_map: &'static str,
    },
    #[error("lookup applied to a map that doesn't contain the given key: {key:?}; map: {map:?}")]
    LookupKeyMissing {
        key: String,
        map: Map<String, Value>,
    },

    #[error("sha256 applied an Elem of an unsupported type ({0})")]
    HashUnsupportedType(&'static str),

    #[error("to_json/from_json serialization failed: ({0})")]
    ToFromJsonFailed(String),

    #[error("from_json applied an Elem of an unsupported type ({0})")]
    FromJsonUnsupportedType(&'static str),

    #[error("object_from_json applied an Elem of an unsupported type ({0})")]
    ObjectFromJsonUnsupportedType(&'static str),
    #[error("object_from_json applied unexpected JSON: ({0})")]
    ObjectFromJsonUnexpecteJson(Value),

    #[error("array_from_json applied an Elem of an unsupported type ({0})")]
    ArrayFromJsonUnsupportedType(&'static str),
    #[error("array_from_json applied unexpected JSON: ({0})")]
    ArrayFromJsonUnexpecteJson(Value),

    #[error("string_from_json applied an Elem of an unsupported type ({0})")]
    StringFromJsonUnsupportedType(&'static str),
    #[error("string_from_json applied unexpected JSON: ({0})")]
    StringFromJsonUnexpecteJson(Value),

    #[error("unpack_json applied to a value that's not raw JSON or it didn't match the expected type: {non_json:?}; type: {elem_symbol:?}")]
    UnpackJsonUnexpectedType {
        non_json: &'static str,
        elem_symbol: &'static str,
    },
    #[error("unpack_json applied to raw JSON and an unsupported type: {json:?}; type: {elem_symbol:?}")]
    UnpackJsonUnsupportedSymbol {
      json: serde_json::Value,
      elem_symbol: &'static str,
    },

    #[error("string_to_bytes applied to an Elem of an unsupported type ({0})")]
    StringToBytesUnsupportedType(&'static str),
}

impl From<serde_json::Error> for ElemError {
    fn from(error: serde_json::Error) -> Self {
        ElemError::ToFromJsonFailed(format!("{}", error))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub enum Instruction {
    Push(Elem),
    Restack(Restack),
    HashSha256,
    CheckLe,
    CheckLt,
    CheckEq,
    Concat,
    Slice,
    Index,
    Lookup,
    AssertTrue,
    ToJson,
    UnpackJson(ElemSymbol),
    StringToBytes,
}

impl Instruction {
    // forall (instr : Instructions) (input_stack: Executor),
    //     if input_stack.len() < instr.stack_io_counts().0 {
    //         stack_too_small error
    //     } else {
    //         output_stack.len() = input_stack.len() - instr.stack_io_counts().0 + instr.stack_io_counts().1
    //     }

    // (consumed_input_stack_size, produced_output_stack_size)
    pub fn stack_io_counts(&self) -> (usize, usize) {
        match self {
            Instruction::Push(_) => (0, 1),
            Instruction::Restack(restack) => restack.stack_io_counts(),
            Instruction::HashSha256 => (1, 1),
            Instruction::CheckLe => (2, 1),
            Instruction::CheckLt => (2, 1),
            Instruction::CheckEq => (2, 1),
            Instruction::Concat => (2, 1),
            Instruction::Slice => (3, 1),
            Instruction::Index => (2, 1),
            Instruction::Lookup => (2, 1),
            Instruction::AssertTrue => (1, 0),
            Instruction::ToJson => (1, 1),
            Instruction::UnpackJson(_) => (1, 1),
            Instruction::StringToBytes => (1, 1),
        }
    }

    // pub fn ty_sets(&self) -> (Vec<Ty>, Vec<Ty>) {
    //     match self {
    //         Instruction::Push(elem) => elem.push_ty_sets(),
    //         // Restack(restack) => restack.ty_sets(),
    //         Instruction::HashSha256 => (vec![ElemSymbol::Bytes.ty()], vec![ElemSymbol::Bytes.ty()]),
    //         // CheckLe,
    //         // CheckLt,
    //         Instruction::CheckEq => (vec![Ty::any(), Ty::any()], vec![ElemSymbol::Bool.ty()]),
    //         // Concat,
    //         // Slice,
    //         // Index,
    //         // Lookup,
    //         // AssertTrue,
    //         // ToJson,
    //         // UnpackJson(ElemSymbol),
    //         // StringToBytes,
    //         _ => panic!("infer_instruction: unimplemented"),
    //     }
    // }
}

pub type Instructions = Vec<Instruction>;
// pub type Stack = Vec<Elem>;

// impl Instructions {
//     pub fn stack_io_counts(&self, input_stack_size: usize) -> (usize, usize) {
//         self.iter().fold((input_stack_size, input_stack_size), |memo, x| {
//             let (memo_input, memo_output) = memo;
//             let (next_input, next_output) = x.stack_io_counts;

//             let mut this_input = memo_input;
//             let mut this_output = memo_output;

//             while this_input < next_input {
//                 this_input += 1;
//                 this_output += 1;
//             }
//             (this_input, this_output + next_output)
//         })
//     }
// }

// Typing Overview:
// - calculate the number of in/out stack elements per instruction
//     + most consume 0..2 and produce one input
//     + exceptions are restack and assert_true
// - trace the stack type variables through the execution
//     + [ instruction ] -> [ (instruction, [stack_variable]) ], num_stack_variables
//     + map from type_var -> [ (instruction_location, (instruction), stack_location) ]
//         * instruction may/may-not be needed here
//         * stack_location differentiates between e.g. index number and iterable
//     + convert to a list of constraints
//     + resolve the list of constraints to a single type














#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SrcRange {
    range: Range<usize>,
}

impl SrcRange {
    pub fn singleton(src_location: usize) -> Self {
        SrcRange {
            range: (src_location..src_location + 1),
        }
    }

    pub fn append(&self, other: Self) -> Result<Self, SrcRangeError> {
        if self.range.end + 1 == other.range.start {
            Ok(SrcRange { range: self.range.start..other.range.end })
        } else {
            Err(SrcRangeError::MismatchedRanges {
                lhs: self.clone(),
                rhs: other,
            })
        }
    }
}

#[derive(Debug, PartialEq, Error)]
pub enum SrcRangeError {
    #[error("SrcRange::append applied to non-contiguous ranges: lhs: {lhs:?}; rhs: {rhs:?}")]
    MismatchedRanges {
        lhs: SrcRange,
        rhs: SrcRange,
    },
}




// #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
// pub struct TyUnifyLocation {
//     lhs: SrcRange,
//     rhs: SrcRange,
//     stack_position: usize,
// }

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Ty {
    ty_set: EnumSet<ElemSymbol>,
}

// impl Ty {
//     pub fn any() -> Self {
//         Ty {
//             ty_set: EnumSet::all(),
//         }
//     }

//     pub fn unify(&self, other: Self, location: TyUnifyLocation) -> Result<Self, TypeError> {
//         let both = self.ty_set.intersection(other.ty_set);
//         if both.is_empty() {
//             Err(TypeError::TyUnifyEmpty {
//                 lhs: self.clone(),
//                 rhs: other,
//                 location: location,
//             })
//         } else {
//             Ok(Ty {
//                 ty_set: both
//             })
//         }
//     }
// }


// typing:
// - inference
// - checking against inferred or other type (this + inference = bidirecitonal)
// - unification
// - two categories of tests:
//   + property tests for typing methods themselves
//   + test that a function having a particular type -> it runs w/o type errors on such inputs

    // Push(Elem),             // (t: type, elem: type(t)) : [] -> [ t ]
    // Restack(Restack),       // (r: restack) : [ .. ] -> [ .. ]
    // HashSha256,             // : [ bytes ] -> [ bytes ]
    // CheckLe,                // : [ x, x ] -> [ bool ]
    // CheckLt,                // : [ x, x ] -> [ bool ]
    // CheckEq,                // : [ x, x ] -> [ bool ]
    // Concat,                 // (t: type, prf: is_concat(t)) : [ t, t ] -> [ t ]
    // Slice,                  // (t: type, prf: is_slice(t)) : [ int, int, t ] -> [ t ]
    // Index,                  // (t: type, prf: is_index(t)) : [ int, t ] -> [ json ]
    // Lookup,                 // [ string, object ] -> [ json ]
    // AssertTrue,             // [ bool ] -> []
    // ToJson,                 // (t: type) : [ t ] -> [ json ]
    // UnpackJson(ElemSymbol), // (t: type) : [ json ] -> [ t ]
    // StringToBytes,          // [ string ] -> [ bytes ]




// // TODO: use in_ty/out_ty

// #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
// pub struct StackTy {
//     src_range: SrcRange,
//     in_ty: Vec<Ty>,
//     out_ty: Vec<Ty>,
// }

// impl StackTy {
//     // pub fn zip_extend_with<T>(xs: Vec<T>, ys: Vec<T>, fl: Fn(T) -> T, f: Fn(T, T)

//     pub fn unify_ty_sets(xs: Vec<Ty>, ys: Vec<Ty>, xs_src_range: SrcRange, ys_src_range: SrcRange) -> Result<Vec<Ty>, TypeError> {
//         let zip_len = cmp::max(xs.len(), ys.len());
//         let xs_extended = xs.iter().map(|z|Some(z)).chain(std::iter::repeat(None).take(zip_len - xs.len()));
//         let ys_extended = ys.iter().map(|z|Some(z)).chain(std::iter::repeat(None).take(zip_len - ys.len()));

//         xs_extended.zip(ys_extended).enumerate().map(|ixy| {
//             match ixy.1 {
//                 (None, None) => Err(TypeError::StackTyUnifyNone {
//                     lhs: xs.clone(),
//                     rhs: ys.clone(),
//                 }),
//                 (Some(x), None) => Ok(*x),
//                 (None, Some(y)) => Ok(*y),
//                 (Some(x), Some(y)) => Ok(x.unify(*y, TyUnifyLocation {
//                     lhs: xs_src_range.clone(),
//                     rhs: ys_src_range.clone(),
//                     stack_position: ixy.0,
//                 })?),
//             }

//         }).collect()
//     }

//     pub fn diff_ty_sets(xs: Vec<Ty>, ys: Vec<Ty>) -> Result<Vec<Ty>, TypeError> {
//         xs.iter().zip(ys.iter()).map(|x, y| {
//             x.difference(y)
//         }.collect()
//     }

//     pub fn union_ty_sets(xs: Vec<Ty>, ys: Vec<Ty>) -> Result<Vec<Ty>, TypeError> {
//         // pad lengths and zip (pad with empty)
//         xs.iter().zip(ys.iter()).map(|x, y| {
//             x.union(y)
//         }.collect()
//     }

//     pub fn unify(&self, other: Self) -> Result<Self, TypeError> {
//         let middle_ty = Self::unify_ty_sets(self.out_ty, other.in_ty, self.src_range, other.src_range);
//         let self_remainder = Self::diff_ty_sets(middle_ty, self.out_ty);
//         let other_remainder = Self::diff_ty_sets(middle_ty, other.in_ty);
//             in_ty: self.in_ty + self_remainder
//             out_ty: other.out_ty + other_remainder

//         StackTy {
//             src_range: self.src_range.append(other.src_range)?,
//             in_ty: Self::union_ty_sets(self.in_ty, self_remainder),
//             out_ty: Self::union_ty_sets(other.out_ty, other_remainder),
//         }
//     }

//     pub fn infer_instruction(instruction: &Instruction, src_location: usize) -> Self {
//         let instruction_ty_sets = instruction.ty_sets();
//         StackTy {
//             src_range: SrcRange::singleton(src_location),
//             in_ty: instruction_ty_sets.0,
//             out_ty: instruction_ty_sets.1,
//         }
//     }

//     // pub fn infer(instructions: Instructions) -> Result<Self, TypeError> {
//     //     instructions.iter().enumerate()
//     //         .map(|ix| Self::infer_instruction(ix.1, ix.0))
//     //         .reduce(|memo, x| memo.unify(x))

//     // }


// }



// #[derive(Debug, PartialEq, Error)]
// pub enum TypeError {
//     #[error("Ty::unify applied to non-intersecting types: lhs: {lhs:?}; rhs: {rhs:?}")]
//     TyUnifyEmpty {
//         lhs: Ty,
//         rhs: Ty,
//         location: TyUnifyLocation,
//     },

//     // should be impossible
//     #[error("StackTy::unify produced an attempt to unify None and None: lhs: {lhs:?}; rhs: {rhs:?}")]
//     StackTyUnifyNone {
//         lhs: Vec<Ty>,
//         rhs: Vec<Ty>,
//     },

//     #[error("attempt to unify types of non-contiguous locations: lhs: {0:?}")]
//     SrcRangeError(SrcRangeError),
// }

// impl From<SrcRangeError> for TypeError {
//     fn from(error: SrcRangeError) -> Self {
//         Self::SrcRangeError(error)
//     }
// }






