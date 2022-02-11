use crate::restack::Restack;

use thiserror::Error;

use std::cmp;
use std::convert::TryFrom;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Number, Value};

// TODO: for demo:
// - step through / display steps better
// - support signatures:
//   + check_sig
//   + to_pub_key

// TODO: property based tests

// TODO: Bool/Number primitives

// Bool(bool),
// - neg
// - and
// - or

// Number(Number), -->> later
// - to_int
// - add
// - sub
// - mul
// - div



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
            (Self::String(x), Self::String(y)) => x.partial_cmp(y),
            (Self::Array(x), Self::Array(y)) => if x == y { Some(cmp::Ordering::Equal) } else { None },
            (Self::Object(x), Self::Object(y)) => if x == y { Some(cmp::Ordering::Equal) } else { None }
            (_, _) => None,
        }
    }
}

impl Elem {
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

    pub fn check_le(&self, other: Self) -> Result<Self, ElemError> {
        let result = match self.partial_cmp(&other)
            .ok_or_else(|| ElemError::CheckLeIncomparableTypes {
                lhs: self.simple_type(),
                rhs: other.simple_type() })? {
                    cmp::Ordering::Less => true,
                    cmp::Ordering::Equal => true,
                    cmp::Ordering::Greater => false,
        };
        Ok(Self::Bool(result))
    }

    pub fn check_lt(&self, other: Self) -> Result<Self, ElemError> {
        let result = match self.partial_cmp(&other)
            .ok_or_else(|| ElemError::CheckLtIncomparableTypes {
                lhs: self.simple_type(),
                rhs: other.simple_type() })? {
                    cmp::Ordering::Less => true,
                    _ => false,
        };
        Ok(Self::Bool(result))
    }

    pub fn check_eq(self, other: Self) -> Result<Self, ElemError> {
        let result = match self.partial_cmp(&other)
            .ok_or_else(|| ElemError::CheckEqIncomparableTypes {
                lhs: self.simple_type(),
                rhs: other.simple_type() })? {
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
                    lhs: &some_x.simple_type(),
                    rhs: &some_y.simple_type()
                })
            },
        }
    }


    fn slice_generic<T: Clone + IntoIterator + std::iter::FromIterator<<T as std::iter::IntoIterator>::Item>>(offset: Number, length: Number, iterable: T) -> Result<T, ElemError> {
        let u_offset = offset.as_u64()
            .ok_or_else(|| ElemError::SliceOffsetNotU64(offset.clone()))
            .and_then(|x| usize::try_from(x).map_err(|_| ElemError::SliceOverflow { offset: offset.clone(), length: length.clone() }))?;
        let u_length = length.as_u64()
            .ok_or_else(|| ElemError::SliceLengthNotU64(length.clone()))
            .and_then(|x| usize::try_from(x).map_err(|_| ElemError::SliceOverflow { offset: offset.clone(), length: length.clone() }))?;
        let u_offset_plus_length = u_offset.checked_add(u_length)
            .ok_or_else(|| ElemError::SliceOverflow { offset: offset.clone(), length: length.clone() })?;
        if iterable.clone().into_iter().count() < u_offset_plus_length {
            panic!("slice_generic SliceTooShort unimplemented")

            // TODO: implement proper error
            // Err(ElemError::SliceTooShort {
            //     offset: u_offset,
            //     length: u_length,
            //     iterable: &Self::Bytes(vec![]).simple_type(),
            // })
        } else {
            Ok(iterable.into_iter().skip(u_offset).take(u_length).collect())
        }
    }

    pub fn slice(maybe_offset: Self, maybe_length: Self, maybe_iterable: Self) -> Result<Self, ElemError> {
        match (maybe_offset, maybe_length, maybe_iterable) {
            (Self::Number(offset), Self::Number(length), Self::Bytes(iterator)) =>
                Ok(Self::Bytes(Self::slice_generic(offset, length, iterator)?)),
            (Self::Number(offset), Self::Number(length), Self::String(iterator)) => {
                let iterator_vec = Vec::from(iterator.clone());
                Ok(Self::String(String::from_utf8(Self::slice_generic(offset.clone(), length.clone(), iterator_vec)?)
                        .map_err(|_| ElemError::SliceInvalidUTF8 { offset: offset, length: length, iterator: iterator })?))
                },
            (Self::Number(offset), Self::Number(length), Self::Array(iterator)) =>
                Ok(Self::Array(Self::slice_generic(offset, length, iterator)?)),
            (Self::Number(offset), Self::Number(length), Self::Object(iterator)) =>
                Ok(Self::Object(Self::slice_generic(offset, length, iterator)?)),
            (maybe_not_offset, maybe_not_length, maybe_not_iterable) => {
                Err(ElemError::SliceUnsupportedTypes {
                    maybe_not_offset: &maybe_not_offset.simple_type(),
                    maybe_not_length: &maybe_not_length.simple_type(),
                    maybe_not_iterable: &maybe_not_iterable.simple_type(),
                })
            }
        }
    }

    fn index_generic<T: Clone + IntoIterator + std::iter::FromIterator<<T as std::iter::IntoIterator>::Item>>(index: Number, iterable: T) -> Result<<T as std::iter::IntoIterator>::Item, ElemError> {
        let u_index: usize = index.as_u64()
            .ok_or_else(|| ElemError::IndexNotU64(index.clone()))
            .and_then(|x| usize::try_from(x).map_err(|_| ElemError::IndexOverflow(index.clone())))?;
        if iterable.clone().into_iter().count() <= u_index {
            return Err(ElemError::IndexTooShort {
                index: u_index,
                iterable: &Self::Bytes(vec![]).simple_type(),
            })
        } else {
            match iterable.into_iter().skip(u_index).next() {
                None => Err(ElemError::IndexTooShort { index: u_index, iterable: &Self::Bytes(vec![]).simple_type() }),
                Some(x) => Ok(x),
            }
        }
    }

    pub fn index(self, maybe_iterable: Self) -> Result<Self, ElemError> {
        match (self, maybe_iterable) {
            // (Self::Number(index), Self::Bytes(iterator)) =>
            //     Ok(Self::Bytes(Self::slice_index(index, length, iterator)?)),
            // (Self::Number(index), Self::String(iterator)) => {
            //     let iterator_vec = Vec::from(iterator.clone());
            //     Ok(Self::String(String::from_utf8(Self::slice_generic(index.clone(), length.clone(), iterator_vec)?)
            //             .map_err(|_| ElemError::SliceInvalidUTF8 { index: index, length: length, iterator: iterator })?))
            //     },
            (Self::Number(index), Self::Array(iterator)) =>
                Ok(Self::Json(Self::index_generic(index, iterator)?)),
            (Self::Number(index), Self::Object(iterator)) =>
                Ok(Self::Json(Self::index_generic(index, iterator)?.1)),
            (maybe_not_index, maybe_not_iterable) => {
                Err(ElemError::IndexUnsupportedTypes {
                    maybe_not_index: &maybe_not_index.simple_type(),
                    maybe_not_iterable: &maybe_not_iterable.simple_type(),
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
                maybe_not_key: &maybe_not_key.simple_type(),
                maybe_not_map: &maybe_not_map.simple_type(),
            }),
        }
    }

    pub fn sha256(self) -> Result<Self, ElemError> {
        match self {
            Self::Bytes(bytes) => {
                Ok(Self::Bytes(super::sha256(&bytes)))
            }
            elem => Err(ElemError::HashUnsupportedType(elem.simple_type())),
        }
    }

    pub fn to_json(self) -> Result<Self, ElemError> {
        Ok(Self::Json(serde_json::to_value(self)?))
    }

    pub fn from_json(self) -> Result<Self, ElemError> {
        match self {
            Self::Json(raw_json) => Ok(serde_json::from_value(raw_json)?),
            non_json => Err(ElemError::FromJsonUnsupportedType(&non_json.simple_type())),
        }
    }

    // pub fn unit_from_json(self) -> Result<Self, ElemError> {
    //     match self {
    //         Self::Json(serde_json::Null) => Ok(Elem::Unit),
    //         other => Err(ElemError::UnitFromJsonUnsupportedType(&other.simple_type())),
    //     }
    // }

    pub fn object_from_json(self) -> Result<Self, ElemError> {
        match self {
            Self::Json(serde_json::Value::Object(x)) => Ok(Elem::Object(x)),
            Self::Json(x) => Err(ElemError::ObjectFromJsonUnexpecteJson(x)),
            other => Err(ElemError::ObjectFromJsonUnsupportedType(&other.simple_type())),
        }
    }

    pub fn array_from_json(self) -> Result<Self, ElemError> {
        match self {
            Self::Json(serde_json::Value::Array(x)) => Ok(Elem::Array(x)),
            Self::Json(x) => Err(ElemError::ArrayFromJsonUnexpecteJson(x)),
            other => Err(ElemError::ArrayFromJsonUnsupportedType(&other.simple_type())),
        }
    }

    pub fn string_from_json(self) -> Result<Self, ElemError> {
        match self {
            Self::Json(serde_json::Value::String(x)) => Ok(Elem::String(x)),
            Self::Json(x) => Err(ElemError::StringFromJsonUnexpecteJson(x)),
            other => Err(ElemError::StringFromJsonUnsupportedType(&other.simple_type())),
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
}

impl From<serde_json::Error> for ElemError {
    fn from(error: serde_json::Error) -> Self {
        ElemError::ToFromJsonFailed(format!("{}", error))
    }
}




// TODO: implement simple_type on this instead of Elem
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ElemSymbol {
    UnitSymbol,
    BoolSymbol,
    NumberSymbol,
    BytesSymbol,
    StringSymbol,
    ArraySymbol,
    ObjectSymbol,
    JsonSymbol,
}

impl From<Elem> for ElemSymbol {
    fn from(x: Elem) -> Self {
        match x {
            Elem::Unit => Self::UnitSymbol,
            Elem::Bool(_) => Self::BoolSymbol,
            Elem::Number(_) => Self::NumberSymbol,
            Elem::Bytes(_) => Self::BytesSymbol,
            Elem::String(_) => Self::StringSymbol,
            Elem::Array(_) => Self::ArraySymbol,
            Elem::Object(_) => Self::ObjectSymbol,
            Elem::Json(_) => Self::JsonSymbol,
        }
    }
}

#[cfg(test)]
impl ElemSymbol {
    // TODO: synchronize with Default trait
    pub fn default_elem(self) -> Elem {
        match self {
            Self::UnitSymbol => Elem::Unit,
            Self::BoolSymbol => Elem::Bool(false),
            Self::NumberSymbol => Elem::Number(From::from(0u8)),
            Self::BytesSymbol => Elem::Bytes(vec![]),
            Self::StringSymbol => Elem::String("".to_string()),
            Self::ArraySymbol => Elem::Array(vec![]),
            Self::ObjectSymbol => Elem::Object(Map::new()),
            Self::JsonSymbol => Elem::Json(serde_json::Value::Null),
        }
    }
}

#[cfg(test)]
mod elem_symbol_tests {
    use super::*;

    #[test]
    fn test_from_default_elem() {
        for symbol in [
            ElemSymbol::UnitSymbol,
            ElemSymbol::BoolSymbol,
            ElemSymbol::NumberSymbol,
            ElemSymbol::BytesSymbol,
            ElemSymbol::StringSymbol,
            ElemSymbol::ArraySymbol,
            ElemSymbol::ObjectSymbol,
            ElemSymbol::JsonSymbol,
        ] {
            assert_eq!(symbol, From::from(symbol.default_elem()))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub enum Instruction {
    Push(Elem),
    FnRestack(Restack),
    FnHashSha256,
    FnCheckLe,
    FnCheckLt,
    FnCheckEq,
    FnConcat,
    FnSlice,
    FnIndex, // Array
    FnLookup, // Map
    FnAssertTrue,
    FnToJson,
    FnFromJson,
    FnObjectFromJson,
    FnArrayFromJson,
    FnStringFromJson,
}

pub type Instructions = Vec<Instruction>;

// pub type Stack = Vec<Elem>;

