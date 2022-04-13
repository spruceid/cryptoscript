use crate::location::{Location};
use crate::elem::{Elem, ElemSymbol};

use thiserror::Error;

use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::iter::{FromIterator, IntoIterator};

use enumset::{EnumSet};
use serde::{Deserialize, Serialize};


/// ElemType metadata
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ElemTypeInfo {
    /// Location of a variable associated with an ElemType
    location: Location,
}

// TODO: make fields private?
/// A set of ElemSymbol's representing a type, with included metadata
///
/// E.g. {String, bool} represents the type that can be a String or a bool.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ElemType {
    /// The set of ElemSymbol's making up this type
    pub type_set: EnumSet<ElemSymbol>,

    /// Type metadata, for debugging, analysis, pretty printing
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
    /// ElemType of a particular ElemSymbol
    pub fn elem_type(&self, locations: Vec<Location>) -> ElemType {
        ElemType {
            type_set: EnumSet::only(*self),
            info: locations.iter()
                .map(|&location|
                     ElemTypeInfo {
                         location: location,
                    }).collect(),
        }
    }
}

impl Elem {
    /// ElemType of a particular Elem. See ElemSymbol::elem_type
    pub fn elem_type(&self, locations: Vec<Location>) -> ElemType {
        self.symbol().elem_type(locations)
    }
}

impl ElemType {
    /// Construct from a type_set and Vec of Location's
    pub fn from_locations(type_set: EnumSet<ElemSymbol>,
                          locations: Vec<Location>) -> Self {
        ElemType {
            type_set: type_set,
            info: locations.iter()
                .map(|&location|
                     ElemTypeInfo {
                         location: location,
                    }).collect(),
        }
    }

    /// The type of any Elem
    pub fn any(locations: Vec<Location>) -> Self {
        Self::from_locations(
            EnumSet::all(),
            locations)
    }

    /// Calculate the union of two ElemType's and append their metadata
    pub fn union(&self, other: Self) -> Self {
        let both = self.type_set.union(other.type_set);
        let mut both_info = self.info.clone();
        both_info.append(&mut other.info.clone());
        ElemType {
            type_set: both,
            info: both_info,
        }
    }

    /// Unify two ElemType's by returning their intersection and combining their metadata
    ///
    /// Fails if their intersection is empty (i.e. if it results in an empty type)
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
    },
}


// TODO: relocate
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

/// The type of a Stack
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StackType {
    /// List of types of the Stack, in the same order as any Stack of this type
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
    /// Length of the StackType, equal to the length of any Stack of this type
    pub fn len(&self) -> usize {
        self.types.len()
    }

    /// Push the given ElemType to the StackType
    pub fn push(&mut self, elem_type: ElemType) -> () {
        self.types.insert(0, elem_type)
    }

    /// Push (count) copies of the given ElemType to the StackType
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

