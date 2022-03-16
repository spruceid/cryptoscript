// use crate::restack::{RestackError};
use crate::elem::{Elem, ElemSymbol};

// Stack, StackError, LineNo, 
use crate::stack::{Location};

use std::collections::BTreeMap;
use std::cmp;

use std::fmt;
use std::fmt::{Display, Formatter};
// use std::alloc::string;
use std::marker::PhantomData;
// use std::sync::Arc;

use enumset::{EnumSet, enum_set};
use serde::{Deserialize, Serialize};
// use serde_json::{Map, Number, Value};
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

// TODO: add necessary traits, methods and rename to IsInstructionError or IsInstrError
pub trait AnError: std::fmt::Debug {
    // fn to_stack_error(&self, line_no: LineNo) -> StackError;
}

// impl StackError {
//     fn instruction_default(name: &str, error_str: &str, line_no: LineNo) -> Self {
//         Self::RunInstruction {
//             name: name.to_string(),
//             error: error_str.to_string(),
//             line_no: line_no,
//         }
//     }
// }

impl AnError for Empty {
    // fn to_stack_error(&self, _line_no: LineNo) -> StackError {
    //     self.absurd(PhantomData)
    // }
}





































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

