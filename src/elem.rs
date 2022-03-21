use crate::arbitrary::{ArbitraryNumber, ArbitraryMap, ArbitraryValue};
use crate::stack::{Location};

use thiserror::Error;

use std::cmp;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
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

// println!("{:x?}", 

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






/* #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)] */
/* pub enum BaseElemType { */
/*     Any, */
/*     Concat, */
/*     Index, */
/*     Slice, */
/*     ElemSymbol(ElemSymbol), */
/* } */

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ElemTypeInfo {
    /* base_elem_type: BaseElemType, */
    location: Location,
}

// TODO: make fields private?
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ElemType {
    pub type_set: EnumSet<ElemSymbol>,
    pub info: Vec<ElemTypeInfo>,
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
                         // base_elem_type: BaseElemType::ElemSymbol(*self),
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
                      // base_elem_type: BaseElemType,
                      locations: Vec<Location>) -> Self {
        ElemType {
            type_set: type_set,
            info: locations.iter()
                .map(|&location|
                     ElemTypeInfo {
                         // base_elem_type: base_elem_type,
                         location: location,
                    }).collect(),
        }
    }

    pub fn any(locations: Vec<Location>) -> Self {
        Self::from_locations(
            EnumSet::all(),
            // BaseElemType::Any,
            locations)
    }

    // pub fn concat_type(locations: Vec<Location>) -> Self {
    //     Self::from_locations(
    //         enum_set!(ElemSymbol::Bytes |
    //                   ElemSymbol::String |
    //                   ElemSymbol::Array |
    //                   ElemSymbol::Object),
    //         // BaseElemType::Concat,
    //         locations)
    // }

    // pub fn index_type(locations: Vec<Location>) -> Self {
    //     Self::from_locations(
    //         enum_set!(ElemSymbol::Array |
    //                   ElemSymbol::Object),
    //         // BaseElemType::Index,
    //         locations)
    // }

    // pub fn slice_type(locations: Vec<Location>) -> Self {
    //     Self::concat_type(locations)
    // }

    pub fn union(&self, other: Self) -> Result<Self, ElemTypeError> {
        let both = self.type_set.union(other.type_set);
        let mut both_info = self.info.clone();
        both_info.append(&mut other.info.clone());
        Ok(ElemType {
            type_set: both,
            info: both_info,
        })
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

#[derive(Clone, Debug, PartialEq, Error)]
pub enum ElemTypeError {
    #[error("ElemType::unify applied to non-intersecting types:\nlhs:\n{lhs}\nrhs:\n{rhs}")]
    UnifyEmpty {
        lhs: ElemType,
        rhs: ElemType,
        // location: TyUnifyLocation,
    },
}



// BEGIN DebugAsDisplay
#[derive(Clone, PartialEq, Eq)]
struct DebugAsDisplay<T>
where
    T: Display,
{
    t: T,
}

impl<T> Display for DebugAsDisplay<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.t)
    }
}

impl<T> Debug for DebugAsDisplay<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.t)
    }
}
// END DebugAsDisplay

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StackType {
    pub types: Vec<ElemType>,
}

impl IntoIterator for StackType {
    type Item = ElemType;
    type IntoIter = <Vec<ElemType> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.types.into_iter()
    }
}

impl FromIterator<ElemType> for StackType {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = ElemType>,
    {
        StackType {
            types: FromIterator::from_iter(iter),
        }
    }
}

impl StackType {
    pub fn len(&self) -> usize {
        self.types.len()
    }

    pub fn push(&mut self, elem_type: ElemType) -> () {
        self.types.insert(0, elem_type)
    }

    pub fn push_n(&mut self, elem_type: ElemType, count: usize) -> () {
        for _index in 0..count {
            self.push(elem_type.clone())
        }
    }
}

// Uses DebugAsDisplay to eliminate '"' around strings:
// ["{Number}", "{Array, Object}"] -> [{Number}, {Array, Object}]
impl Display for StackType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_list()
            .entries(self.types
                     .iter()
                     .map(|x| DebugAsDisplay { t: format!("{}", x) }))
            .finish()?;
        Ok(())
    }
}






pub trait AnElem: Clone + std::fmt::Debug + PartialEq {
    // TODO: rename

    // fn elem_symbol(t: PhantomData<Self>) -> ElemType;
    fn elem_symbol(t: PhantomData<Self>) -> EnumSet<ElemSymbol>;
    fn to_elem(self) -> Elem;
    fn from_elem(t: PhantomData<Self>, x: Elem) -> Result<Self, AnElemError>;
}

impl AnElem for Elem {
    fn elem_symbol(_t: PhantomData<Self>) -> EnumSet<ElemSymbol> {
        EnumSet::all()
    }

    fn to_elem(self) -> Elem {
        self
    }

    fn from_elem(_t: PhantomData<Self>, x: Elem) -> Result<Self, AnElemError> {
        Ok(x)
    }
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
    #[error("AnElem::from_elem: element popped from the stack\n\n{found}\n\nwasn't the expected type:\n{expected:?}")]
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

