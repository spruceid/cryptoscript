use crate::arbitrary::{ArbitraryNumber, ArbitraryMap, ArbitraryValue};

use std::cmp;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::iter::IntoIterator;

use enumset::{EnumSet, EnumSetType};
use quickcheck::{empty_shrinker, Arbitrary, Gen};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Number, Value};

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

impl Display for Elem {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::Unit => write!(f, "()"),
            Self::Bool(x) => write!(f, "{}", x),
            Self::Number(x) => write!(f, "{}", x),
            Self::Bytes(x) => write!(f, "{}", hex::encode(x.as_slice())),
            Self::String(x) => write!(f, "{}", x),
            Self::Array(x) => {
                f.debug_list()
                    .entries(x.iter()
                             .map(|x| format!("{}", x)))
                    .finish()
            },
            Self::Object(x) => {
                f.debug_list()
                    .entries(x.iter()
                             .map(|(x, y)| format!("({}, {})", x.clone(), y.clone())))
                    .finish()
            },
            Self::Json(x) => write!(f, "{}", x),
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

    // TODO: shrink
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

