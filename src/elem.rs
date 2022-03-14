use crate::arbitrary::{ArbitraryNumber, ArbitraryMap, ArbitraryValue};

use thiserror::Error;

use std::cmp;
use std::convert::TryFrom;
use std::marker::PhantomData;
use std::iter::{FromIterator, IntoIterator};

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
}










pub trait AnElem: Clone + std::fmt::Debug + PartialEq {
    fn elem_symbol(t: PhantomData<Self>) -> EnumSet<ElemSymbol>;
    fn to_elem(self) -> Elem;
    fn from_elem(t: PhantomData<Self>, x: Elem) -> Result<Self, AnElemError>;
}

impl AnElem for () {
    fn elem_symbol(_t: PhantomData<Self>) -> EnumSet<ElemSymbol> {
        EnumSet::only(ElemSymbol::Unit)
    }

    fn to_elem(self) -> Elem {
        Elem::Unit
    }

    fn from_elem(_t: PhantomData<Self>, x: Elem) -> Result<Self, AnElemError> {
        let elem_symbol = <Self as AnElem>::elem_symbol(PhantomData);
        match x {
            Elem::Unit => Ok(()),
            _ => Err(AnElemError::UnexpectedElemType {
                expected: elem_symbol,
                found: x,
            }),
        }
    }
}

impl AnElem for bool {
    fn elem_symbol(_t: PhantomData<Self>) -> EnumSet<ElemSymbol> {
        EnumSet::only(ElemSymbol::Bool)
    }

    fn to_elem(self) -> Elem {
        Elem::Bool(self)
    }

    fn from_elem(_t: PhantomData<Self>, x: Elem) -> Result<Self, AnElemError> {
        let elem_symbol = <Self as AnElem>::elem_symbol(PhantomData);
        match x {
            Elem::Bool(y) => Ok(y),
            _ => Err(AnElemError::UnexpectedElemType {
                expected: elem_symbol,
                found: x,
            }),
        }
    }
}

impl AnElem for Number {
    fn elem_symbol(_t: PhantomData<Self>) -> EnumSet<ElemSymbol> {
        EnumSet::only(ElemSymbol::Number)
    }

    fn to_elem(self) -> Elem {
        Elem::Number(self)
    }

    fn from_elem(_t: PhantomData<Self>, x: Elem) -> Result<Self, AnElemError> {
        let elem_symbol = <Self as AnElem>::elem_symbol(PhantomData);
        match x {
            Elem::Number(y) => Ok(y),
            _ => Err(AnElemError::UnexpectedElemType {
                expected: elem_symbol,
                found: x,
            }),
        }
    }
}

impl AnElem for Vec<u8> {
    fn elem_symbol(_t: PhantomData<Self>) -> EnumSet<ElemSymbol> {
        EnumSet::only(ElemSymbol::Bytes)
    }

    fn to_elem(self) -> Elem {
        Elem::Bytes(self)
    }

    fn from_elem(_t: PhantomData<Self>, x: Elem) -> Result<Self, AnElemError> {
        let elem_symbol = <Self as AnElem>::elem_symbol(PhantomData);
        match x {
            Elem::Bytes(y) => Ok(y),
            _ => Err(AnElemError::UnexpectedElemType {
                expected: elem_symbol,
                found: x,
            }),
        }
    }
}

impl AnElem for String {
    fn elem_symbol(_t: PhantomData<Self>) -> EnumSet<ElemSymbol> {
        EnumSet::only(ElemSymbol::String)
    }

    fn to_elem(self) -> Elem {
        Elem::String(self)
    }

    fn from_elem(_t: PhantomData<Self>, x: Elem) -> Result<Self, AnElemError> {
        let elem_symbol = <Self as AnElem>::elem_symbol(PhantomData);
        match x {
            Elem::String(y) => Ok(y),
            _ => Err(AnElemError::UnexpectedElemType {
                expected: elem_symbol,
                found: x,
            }),
        }
    }
}

impl AnElem for Vec<Value> {
    fn elem_symbol(_t: PhantomData<Self>) -> EnumSet<ElemSymbol> {
        EnumSet::only(ElemSymbol::Array)
    }

    fn to_elem(self) -> Elem {
        Elem::Array(self)
    }

    fn from_elem(_t: PhantomData<Self>, x: Elem) -> Result<Self, AnElemError> {
        let elem_symbol = <Self as AnElem>::elem_symbol(PhantomData);
        match x {
            Elem::Array(y) => Ok(y),
            _ => Err(AnElemError::UnexpectedElemType {
                expected: elem_symbol,
                found: x,
            }),
        }
    }
}

impl AnElem for Map<String, Value> {
    fn elem_symbol(_t: PhantomData<Self>) -> EnumSet<ElemSymbol> {
        EnumSet::only(ElemSymbol::Object)
    }

    fn to_elem(self) -> Elem {
        Elem::Object(self)
    }

    fn from_elem(_t: PhantomData<Self>, x: Elem) -> Result<Self, AnElemError> {
        let elem_symbol = <Self as AnElem>::elem_symbol(PhantomData);
        match x {
            Elem::Object(y) => Ok(y),
            _ => Err(AnElemError::UnexpectedElemType {
                expected: elem_symbol,
                found: x,
            }),
        }
    }
}

impl AnElem for Value {
    fn elem_symbol(_t: PhantomData<Self>) -> EnumSet<ElemSymbol> {
        EnumSet::only(ElemSymbol::Json)
    }

    fn to_elem(self) -> Elem {
        Elem::Json(self)
    }

    fn from_elem(_t: PhantomData<Self>, x: Elem) -> Result<Self, AnElemError> {
        let elem_symbol = <Self as AnElem>::elem_symbol(PhantomData);
        match x {
            Elem::Json(y) => Ok(y),
            _ => Err(AnElemError::UnexpectedElemType {
                expected: elem_symbol,
                found: x,
            }),
        }
    }
}


#[derive(Clone, Debug, Error)]
pub enum AnElemError {
    #[error("AnElem::from_elem: element popped from the stack {found:?} wasn't the expected type {expected:?}")]
    UnexpectedElemType {
        expected: EnumSet<ElemSymbol>,
        found: Elem,
    },

    #[error("<Or<_, _> as AnElem>::from_elem: {e_hd:?}\n{e_tl:?}")]
    PopOr {
        e_hd: Box<Self>,
        e_tl: Box<Self>,
    },
}

