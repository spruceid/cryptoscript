use crate::arbitrary::{ArbitraryNumber, ArbitraryMap, ArbitraryValue};

use thiserror::Error;

use std::cmp;
use std::convert::TryFrom;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Number, Value};

use enumset::{EnumSet, EnumSetType};
use quickcheck::{empty_shrinker, Arbitrary, Gen};

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
            (Self::Number(x), Self::Number(y)) => x.to_string().partial_cmp(&y.to_string()),
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
            Self::Array => {
                let xs: Vec<ArbitraryValue> = Arbitrary::arbitrary(g);
                Elem::Array(xs.into_iter().map(|x| x.value).collect())
            },
            Self::Object => {
                let xs: ArbitraryMap = Arbitrary::arbitrary(g);
                Elem::Object(From::from(xs))
            },
            Self::Json => {
                let xs: ArbitraryValue = Arbitrary::arbitrary(g);
                Elem::Json(xs.value)
            },
        }
    }
}

impl Arbitrary for Elem {
    fn arbitrary(g: &mut Gen) -> Self {
        let symbol: ElemSymbol = Arbitrary::arbitrary(g);
        symbol.arbitrary_contents(g)
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        empty_shrinker()
        // let self_copy = self.clone();
        // Box::new(EnumSet::all().iter().filter(move |&x| x < self_copy))
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
            Self::Number => Elem::Number(From::<u8>::from(Default::default())),
            Self::Bytes => Elem::Bytes(Default::default()),
            Self::String => Elem::String(Default::default()),
            Self::Array => Elem::Array(Default::default()),
            Self::Object => Elem::Object(Default::default()),
            Self::Json => Elem::Json(Default::default()),
        }
    }
}

#[cfg(test)]
mod elem_symbol_tests {
    use super::*;

    #[test]
    fn test_from_default_elem() {
        for symbol in EnumSet::<ElemSymbol>::all().iter() {
            assert_eq!(symbol, symbol.default_elem().symbol())
        }
    }

    #[test]
    fn test_to_default_elem() {
        for default_elem in [
          Elem::Unit,
          Elem::Bool(Default::default()),
          Elem::Number(From::<u8>::from(Default::default())),
          Elem::Bytes(Default::default()),
          Elem::String(Default::default()),
          Elem::Array(Default::default()),
          Elem::Object(Default::default()),
          Elem::Json(Default::default()),
        ] {
            assert_eq!(default_elem, default_elem.symbol().default_elem())
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

