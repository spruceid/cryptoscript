use crate::restack::{Restack, RestackError};
use crate::elem::{Elem, ElemSymbol};

use std::collections::BTreeMap;
use std::cmp;
use std::fmt;
use std::fmt::{Display, Formatter};
// use std::alloc::string;
use std::marker::PhantomData;

use dyn_clone::DynClone;
use enumset::{EnumSet, enum_set};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// typing:
// - unification
//   + inference
//   + checking against inferred or other type (this + inference = bidirecitonal)
// - two categories of tests:
//   + property tests for typing methods themselves
//   + test that a function having a particular type -> it runs w/o type errors on such inputs

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LineNo {
    line_no: usize,
}

impl From<usize> for LineNo {
    fn from(line_no: usize) -> Self {
        LineNo {
            line_no: line_no,
        }
    }
}

pub type ArgumentIndex = usize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Location {
    line_no: LineNo,
    argument_index: ArgumentIndex,
    is_input: bool,
}

impl LineNo {
    pub fn in_at(&self, argument_index: usize) -> Location {
        Location {
            line_no: *self,
            argument_index: argument_index,
            is_input: true,
        }
    }

    pub fn out_at(&self, argument_index: usize) -> Location {
        Location {
            line_no: *self,
            argument_index: argument_index,
            is_input: false,
        }
    }
}

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
    context: Context,
    i_type: Vec<TypeId>,
    o_type: Vec<TypeId>,
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



impl Restack {
    // TODO: fix locations: out locations are mislabeled as in locations
    pub fn type_of(&self, line_no: LineNo) -> Result<Type, RestackError> {
        let mut context = Context::new();
        let mut restack_type: Vec<TypeId> = (0..self.restack_depth)
            .map(|x| context.push(ElemType::any(vec![line_no.in_at(x)])))
            .collect();
        Ok(Type {
            context: context,
            i_type: restack_type.clone(),
            o_type: self.run(&mut restack_type)?,
        })
    }
}

/// Push(Elem),             // (t: type, elem: type(t)) : [] -> [ t ]
/// Restack(Restack),       // (r: restack) : [ .. ] -> [ .. ]
/// HashSha256,             // : [ bytes ] -> [ bytes ]
/// CheckLe,                // : [ x, x ] -> [ bool ]
/// CheckLt,                // : [ x, x ] -> [ bool ]
/// CheckEq,                // : [ x, x ] -> [ bool ]
/// Concat,                 // (t: type, prf: is_concat(t)) : [ t, t ] -> [ t ]
/// Slice,                  // (t: type, prf: is_slice(t)) : [ int, int, t ] -> [ t ]
/// Index,                  // (t: type, prf: is_index(t)) : [ int, t ] -> [ json ]
/// Lookup,                 // [ string, object ] -> [ json ]
/// AssertTrue,             // [ bool ] -> []
/// ToJson,                 // (t: type) : [ t ] -> [ json ]
/// UnpackJson(ElemSymbol), // (t: type) : [ json ] -> [ t ]
/// StringToBytes,          // [ string ] -> [ bytes ]
impl Instruction {
    pub fn type_of(&self, line_no: LineNo) -> Result<Type, TypeError> {
        match self {
            Instruction::Restack(restack) =>
                Ok(restack
                   .type_of(line_no)
                   .or_else(|e| Err(TypeError::InstructionTypeOfRestack(e)))?),

            Instruction::AssertTrue => {
                let mut context = Context::new();
                let bool_var = context
                    .push(ElemSymbol::Bool
                          .elem_type(vec![line_no.in_at(0)]));
                Ok(Type {
                    context: context,
                    i_type: vec![bool_var],
                    o_type: vec![],
                })
            },

            Instruction::Push(elem) => {
                let mut context = Context::new();
                let elem_var = context
                    .push(elem.elem_type(vec![line_no.out_at(0)]));
                Ok(Type {
                    context: context,
                    i_type: vec![],
                    o_type: vec![elem_var],
                })
            },

            Instruction::HashSha256 => {
                let mut context = Context::new();
                let bytes_var = context.push(ElemSymbol::Bytes.elem_type(vec![line_no.in_at(0), line_no.out_at(0)]));
                Ok(Type {
                    context: context,
                    i_type: vec![bytes_var],
                    o_type: vec![bytes_var],
                })
            },

            Instruction::ToJson => {
                let mut context = Context::new();
                let any_var = context.push(ElemType::any(vec![line_no.in_at(0)]));
                let json_var = context.push(ElemSymbol::Json.elem_type(vec![line_no.out_at(0)]));
                Ok(Type {
                    context: context,
                    i_type: vec![any_var],
                    o_type: vec![json_var],
                })
            },

            Instruction::StringToBytes => {
                let mut context = Context::new();
                let string_var = context.push(ElemSymbol::String.elem_type(vec![line_no.in_at(0)]));
                let bytes_var = context.push(ElemSymbol::Bytes.elem_type(vec![line_no.out_at(0)]));
                Ok(Type {
                    context: context,
                    i_type: vec![string_var],
                    o_type: vec![bytes_var],
                })
            },

            Instruction::UnpackJson(elem_symbol) => {
                let mut context = Context::new();
                let json_var = context.push(ElemSymbol::Json.elem_type(vec![line_no.in_at(0)]));
                let elem_symbol_var = context.push(elem_symbol.elem_type(vec![line_no.out_at(0)]));
                Ok(Type {
                    context: context,
                    i_type: vec![json_var],
                    o_type: vec![elem_symbol_var],
                })
            },

            Instruction::CheckLe => {
                let mut context = Context::new();
                let any_lhs_var = context.push(ElemType::any(vec![line_no.in_at(0)]));
                let any_rhs_var = context.push(ElemType::any(vec![line_no.in_at(1)]));
                let bool_var = context.push(ElemSymbol::Bool.elem_type(vec![line_no.out_at(0)]));
                Ok(Type {
                    context: context,
                    i_type: vec![any_lhs_var, any_rhs_var],
                    o_type: vec![bool_var],
                })
            },

            Instruction::CheckLt => {
                let mut context = Context::new();
                let any_lhs_var = context.push(ElemType::any(vec![line_no.in_at(0)]));
                let any_rhs_var = context.push(ElemType::any(vec![line_no.in_at(1)]));
                let bool_var = context.push(ElemSymbol::Bool.elem_type(vec![line_no.out_at(0)]));
                Ok(Type {
                    context: context,
                    i_type: vec![any_lhs_var, any_rhs_var],
                    o_type: vec![bool_var],
                })
            },

            Instruction::CheckEq => {
                let mut context = Context::new();
                let any_lhs_var = context.push(ElemType::any(vec![line_no.in_at(0)]));
                let any_rhs_var = context.push(ElemType::any(vec![line_no.in_at(1)]));
                let bool_var = context.push(ElemSymbol::Bool.elem_type(vec![line_no.out_at(0)]));
                Ok(Type {
                    context: context,
                    i_type: vec![any_lhs_var, any_rhs_var],
                    o_type: vec![bool_var],
                })
            },

            Instruction::Concat => {
                let mut context = Context::new();
                let concat_var = context.push(ElemType::concat_type(vec![line_no.in_at(0), line_no.in_at(1), line_no.out_at(0)]));
                Ok(Type {
                    context: context,
                    i_type: vec![concat_var, concat_var],
                    o_type: vec![concat_var],
                })
            },

            Instruction::Index => {
                let mut context = Context::new();
                let number_var = context.push(ElemSymbol::Number.elem_type(vec![line_no.in_at(0)]));
                let index_var = context.push(ElemType::index_type(vec![line_no.in_at(1), line_no.out_at(0)]));
                Ok(Type {
                    context: context,
                    i_type: vec![number_var, index_var],
                    o_type: vec![index_var],
                })
            },

            Instruction::Lookup => {
                let mut context = Context::new();
                let string_var = context.push(ElemSymbol::String.elem_type(vec![line_no.in_at(0)]));
                let object_var = context.push(ElemSymbol::Object.elem_type(vec![line_no.in_at(1), line_no.out_at(0)]));
                Ok(Type {
                    context: context,
                    i_type: vec![string_var, object_var],
                    o_type: vec![object_var],
                })
            },

            Instruction::Slice => {
                let mut context = Context::new();
                let offset_number_var = context.push(ElemSymbol::Number.elem_type(vec![line_no.in_at(0)]));
                let length_number_var = context.push(ElemSymbol::Number.elem_type(vec![line_no.in_at(1)]));
                let slice_var = context.push(ElemType::slice_type(vec![line_no.in_at(2), line_no.out_at(0)]));
                Ok(Type {
                    context: context,
                    i_type: vec![offset_number_var, length_number_var, slice_var],
                    o_type: vec![slice_var],
                })
            },
        }.or_else(|e| Err(TypeError::InstructionTypeOfDetail {
            instruction: self.clone(),
            error: Box::new(e),
        }))
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


    #[error("Instruction::type_of resulted in restack error: {0:?}")]
    InstructionTypeOfRestack(RestackError),

    #[error("Instruction::type_of resulted in an error involving: {instruction:?};\n {error:?}")]
    InstructionTypeOfDetail {
        instruction: Instruction,
        error: Box<Self>,
    },

    #[error("Instructions::type_of called on an empty Vec of Instruction's")]
    InstructionsTypeOfEmpty,

    #[error("Instructions::type_of resulted in an error on line: {line_no:?};\n {error:?}")]
    InstructionsTypeOfLineNo {
        line_no: usize,
        error: Box<Self>,
    },

    #[error("Type::normalize applying TypeIdMap failed: {0:?}")]
    TypeIdMapError(TypeIdMapError),
}



// pub type Stack = Vec<Elem>;
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub struct Instructions {
    pub instructions: Vec<Instruction>,
}

impl IntoIterator for Instructions {
    type Item = Instruction;
    type IntoIter = <Vec<Instruction> as std::iter::IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.instructions.into_iter()
    }
}

impl Instructions {
    pub fn type_of(&self) -> Result<Type, TypeError> {
        let mut current_type = Type::id();
        for (i, instruction) in self.instructions.iter().enumerate() {
            current_type = current_type.compose(instruction.type_of(From::from(i + 1))?)
                .or_else(|e| Err(TypeError::InstructionsTypeOfLineNo { // TODO: deprecated by Location
                    line_no: i,
                    error: Box::new(e),
                }))?;

            println!("line {i}: {current_type}", i = i, current_type = current_type);
        }
        Ok(current_type)
    }
}

// Test program #1: [] -> []
//
// Instruction::Push(Elem::Bool(true)),
// Instruction::Restack(Restack::id()),
// Instruction::AssertTrue,

// Test program #2
//
// ∀ (t0 ∊ {JSON}),
// ∀ (t1 ∊ {JSON}),
// ∀ (t2 ∊ {Object}),
// [t1] ->
// [t0, t2, t1]
//
// Instruction::Push(Elem::Json(Default::default())),
// Instruction::UnpackJson(ElemSymbol::Object),
// Instruction::Restack(Restack::dup()),

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


#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TUnit {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TBool {
    get_bool: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TNumber {
    number: serde_json::Number,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TBytes {
    bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TString {
    string: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TArray {
    array: Vec<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TObject {
    object: serde_json::Map<String, serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TJson {
    json: serde_json::Value,
}





// Deriving a string from the type without any values allows debugging TEq
pub trait TypeName {
    fn type_name(x: PhantomData<Self>) -> &'static str;
}

impl TypeName for TUnit {
    fn type_name(_: PhantomData<Self>) -> &'static str {
        "Unit"
    }
}


pub trait Teq<T>: DynClone {
    type Other;

    fn transport(&self, x: T) -> Self::Other;
}

impl<T, U: Clone> Teq<T> for U {
    type Other = T;

    fn transport(&self, x: T) -> Self::Other {
        x
    }
}

// dyn_clone::clone_trait_object!(Teq);

// #[derive(Clone)]
pub struct TEq<T, U>
{
    teq: Box<dyn Teq<T, Other = U>>,
}

impl<T, U> PartialEq for TEq<T, U> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<T, U> Eq for TEq<T, U> {}

impl<T, U> TEq<T, U> {
    pub fn transport(&self, x: T) -> U {
        (*self.teq).transport(x)
    }

    // // TODO: compose
    // pub fn compose<V>(&self, uv: TEq<U, V>) -> TEq<T, V> {
    //     TEq {
    //         teq: Box::new(()),
    //     }
    // }
}

impl<T> TEq<T, T> {
    pub fn refl(_x: PhantomData<T>) -> Self {
        TEq {
            teq: Box::new(()),
        }
    }
}

impl<T: TypeName, U: TypeName> fmt::Debug for TEq<T, U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "Teq {{ teq: Teq<{0}, {1}> }}", TypeName::type_name(PhantomData::<T>), TypeName::type_name(PhantomData::<T>))
    }
}

impl<T, U> Clone for TEq<T, U> {
    fn clone(&self) -> Self {
        TEq {
            teq: dyn_clone::clone_box(&*self.teq),
        }

        // self.compose(TEq::refl(PhantomData::<U>))
    }
}


#[derive(Clone, PartialEq, Eq)]
pub enum IsElem<T> {
    Unit(TEq<T, TUnit>),
    Bool(TEq<T, TBool>),
    // Number(TEq<T, serde_json::Number>),
    // Bytes(TEq<T, Vec<u8>>),
    // String(TEq<T, String>),
    // Array(TEq<T, Vec<serde_json::Value>>),
    // Object(TEq<T, serde_json::Map<String, serde_json::Value>>),
    // Json(TEq<T, serde_json::Value>),
}

impl<T: TypeName> fmt::Debug for IsElem<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "IsElem {}", TypeName::type_name(PhantomData::<T>))
    }
}

// impl<T: TypeName, U: TypeName> fmt::Debug for TEq<T, U> {
//     fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
//         write!(f, "Teq {{ teq: Teq<{0}, {1}> }}", TypeName::type_name(PhantomData<T>), TypeName::type_name(PhantomData<T>))
//     }
// }




impl<T> IsElem<T> {
    pub fn elem_symbol(&self) -> ElemSymbol {
        match self {
            Self::Unit(_) => ElemSymbol::Unit,
            Self::Bool(_) => ElemSymbol::Bool,
            // Self::Number(_) => ElemSymbol::Number,
            // Self::Bytes(_) => ElemSymbol::Bytes,
            // Self::String(_) => ElemSymbol::String,
            // Self::Array(_) => ElemSymbol::Array,
            // Self::Object(_) => ElemSymbol::Object,
            // Self::Json(_) => ElemSymbol::Json,
        }
    }

    pub fn to_elem(&self, x: T) -> Elem {
        match self {
            Self::Unit(_) => Elem::Unit,
            Self::Bool(eq) => Elem::Bool(eq.transport(x).get_bool),
            // Self::Number(eq) => Elem::Number(eq.transport(x)),
            // Self::Bytes(eq) => Elem::Bytes(eq.transport(x)),
            // Self::String(eq) => Elem::String(eq.transport(x)),
            // Self::Array(eq) => Elem::Array(eq.transport(x)),
            // Self::Object(eq) => Elem::Object(eq.transport(x)),
            // Self::Json(eq) => Elem::Json(eq.transport(x)),
        }
    }

    // TODO: from_elem
    // fn from_elem(x: &Elem) -> Option<Self> where Self: Sized;
}


// fn fold<B, F>(&self, init: B, f: F) -> B where F: Fn(B, Elem) -> B;
pub trait HList: Clone + IntoIterator<Item = Elem> {
    type Hd;
    type Tl: HList;

    // fn is_empty(&self) -> bool;

    fn hd(&self) -> Self::Hd;
    fn hd_is_elem(&self) -> IsElem<Self::Hd>;

    // fn tl(&self) -> Self::Tl;
    // fn cons<T>(&self, x: T, is_elem: IsElem<T>) -> Cons<T, Self> where Self: Sized;

}



#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Nil { }

impl Iterator for Nil {
    type Item = Elem;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

impl HList for Nil {
    type Hd = TUnit;
    type Tl = Nil;

    // fn is_empty(&self) -> bool {
    //     true
    // }

    fn hd(&self) -> Self::Hd {
        TUnit {}
    }

    fn hd_is_elem(&self) -> IsElem<Self::Hd> {
        IsElem::Unit(TEq::refl(PhantomData::<Self::Hd>))
    }

    // fn tl(&self) -> Self::Tl {
    //     (*self).clone()
    // }

    // fn cons<U: Trait<T>>(&self, x: U) -> Cons<T, U, Self>
    // where
    //     Self: Sized,
    // {
    //     Cons {
    //         t: PhantomData,
    //         hd: x,
    //         tl: (*self).clone(),
    //     }
    // }
}


#[derive(Clone, PartialEq, Eq)]
pub struct Cons<T, U: HList> {
    is_elem: IsElem<T>,
    hd: T,
    tl: U,
}

#[derive(Clone, PartialEq, Eq)]
pub struct IterCons<T, U: HList> {
    cons: Cons<T, U>,
    at_head: bool,
}

impl<T: Copy, U: HList> IntoIterator for Cons<T, U> {
    type Item = Elem;
    type IntoIter = IterCons<T, U>;

    fn into_iter(self) -> Self::IntoIter {
        IterCons {
            cons: self,
            at_head: true,
        }
    }
}

impl<T: Copy, U: HList> Iterator for IterCons<T, U> {
    type Item = Elem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.at_head {
            Some(self.cons.hd_is_elem().to_elem(self.cons.hd))
        } else {
            let self_cons = self.cons.clone();
            *self = self_cons.into_iter();
            self.next()
        }
    }
}

impl<T: Copy, U: HList> HList for Cons<T, U> {
    type Hd = T;
    type Tl = U;

    // fn is_empty(&self) -> bool {
    //     false
    // }

    fn hd(&self) -> Self::Hd {
        self.hd
    }

    fn hd_is_elem(&self) -> IsElem<Self::Hd> {
        self.is_elem.clone()
    }

    // fn tl(&self) -> Self::Tl {
    //     self.tl.clone()
    // }

    // fn cons<W: Trait<T>>(&self, x: W) -> Cons<T, W, Self> {
    //     Cons {
    //         t: PhantomData,
    //         hd: x,
    //         tl: (*self).clone(),
    //     }
    // }

}

// pub fn demo_triple() -> Cons<(), TBool, Cons<(), TUnit, Cons<(), TBool, Nil<()>>>> {
//     Nil { t: PhantomData }
//     .cons(TBool { get_bool: true })
//     .cons(TUnit { })
//     .cons(TBool { get_bool: false })
// }

// pub fn demo_triple_with_tl_handles_intermediate_types() -> Cons<(), TBool, Cons<(), TUnit, Cons<(), TBool, Nil<()>>>> {
//     Nil { t: PhantomData }
//     .cons(TBool { get_bool: true })
//     .cons(TUnit { })
//     .cons(TBool { get_bool: false })
//     .cons(TBool { get_bool: true })
//     .cons(TUnit { })
//     .tl()
//     .tl()
// }

