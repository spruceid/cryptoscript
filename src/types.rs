use crate::restack::{Restack, RestackError};
use crate::elem::{Elem, ElemSymbol};

use std::collections::BTreeMap;
use std::cmp;
use std::iter::Skip;
use std::fmt;
use std::fmt::{Display, Formatter};

use enumset::{EnumSet, enum_set};
use serde::{Deserialize, Serialize};
use thiserror::Error;


// TODO: relocate
pub fn after_zip<A, B>(a: A, b: B) -> Result<Skip<<A as std::iter::IntoIterator>::IntoIter>, Skip<<B as std::iter::IntoIterator>::IntoIter>>
where
    A: IntoIterator,
    B: IntoIterator,
    <A as std::iter::IntoIterator>::IntoIter: ExactSizeIterator,
    <B as std::iter::IntoIterator>::IntoIter: ExactSizeIterator,
{
    let a_iter = a.into_iter();
    let b_iter = b.into_iter();
    let max_len = cmp::max(a_iter.len(), b_iter.len());
    if max_len == a_iter.len() {
        Ok(a_iter.skip(b_iter.len()))
    } else {
        Err(b_iter.skip(a_iter.len()))
    }
}

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

// typing:
// - inference
// - checking against inferred or other type (this + inference = bidirecitonal)
// - unification
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
            assert_eq!(format!("{{{}}}", Into::<&'static str>::into(elem_symbol)), format!("{}", elem_type));
        }
    }

    #[test]
    fn test_all() {
        assert_eq!("{Unit, Bool, Number, Bytes, String, Array, Object, JSON}", format!("{}", ElemType::any(vec![])));
    }
}

impl ElemSymbol {
    pub fn elem_type(&self, locations: Vec<Location>) -> ElemType {
        ElemType {
            type_set: EnumSet::only(*self),
            info: locations.iter()
                .map(|location|
                     ElemTypeInfo {
                         base_elem_type: BaseElemType::ElemSymbol(*self),
                         location: *location,
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
    pub fn any(locations: Vec<Location>) -> Self {
        ElemType {
            type_set: EnumSet::all(),
            info: locations.iter()
                .map(|location|
                     ElemTypeInfo {
                         base_elem_type: BaseElemType::Any,
                         location: *location,
                    }).collect(),
        }
    }

    pub fn concat_type(locations: Vec<Location>) -> Self {
        ElemType {
            type_set:
                enum_set!(ElemSymbol::Bytes |
                          ElemSymbol::String |
                          ElemSymbol::Array |
                          ElemSymbol::Object),
            info: locations.iter()
                .map(|location|
                     ElemTypeInfo {
                         base_elem_type: BaseElemType::Concat,
                         location: *location,
                    }).collect(),
        }
    }

    pub fn index_type(locations: Vec<Location>) -> Self {
        ElemType {
            type_set:
                enum_set!(ElemSymbol::Array |
                          ElemSymbol::Object),
            info: locations.iter()
                .map(|location|
                     ElemTypeInfo {
                         base_elem_type: BaseElemType::Index,
                         location: *location,
                    }).collect(),
        }
    }

    pub fn slice_type(locations: Vec<Location>) -> Self {
        Self::concat_type(locations)
    }

    pub fn unify(&self, other: Self) -> Result<Self, TypeError> {
        let both = self.type_set.intersection(other.type_set);
        if both.is_empty() {
            Err(TypeError::ElemTypeUnifyEmpty {
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

    pub fn new_max(&self, other: Self) -> Self {
        Context {
            context: BTreeMap::new(),
            next_type_id: TypeId {
                type_id:
                    cmp::max(self.next_type_id.type_id,
                             other.next_type_id.type_id),
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

    pub fn get(&mut self, index: &TypeId) -> Result<ElemType, TypeError> {
        Ok(self.context.get(index).ok_or_else(|| TypeError::ContextGetUnknownTypeId {
            context: self.clone(),
            index: *index,
        })?.clone())
    }

    // pub fn push_unified(&mut self, xs: Self, ys: Self, xi: TypeId, yi: TypeId) -> Self {
    //     _
    // }

    // TODO: remove mut xs/ys
    pub fn unify(&mut self, mut xs: Self, mut ys: Self, xi: &TypeId, yi: &TypeId) -> Result<TypeId, TypeError> {
        let x_type = xs.get(xi)?;
        let y_type = ys.get(yi)?;
        Ok(self.push(x_type.unify(y_type)?))
    }
}



#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Type {
    context: Context,
    i_type: Vec<TypeId>,
    o_type: Vec<TypeId>,
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
        write!(f,
               "{context}\n[{i_type}] ->\n[{o_type}]",
               context = self.context,
               i_type = self.i_type.iter().fold(String::new(), |memo, x| {
                   let x_str = format!("t{}", x.type_id);
                   if memo == "" {
                       x_str
                   } else {
                       memo + ", " + &x_str.to_string()
                   }}),
               o_type = self.o_type.iter().fold(String::new(), |memo, x| {
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

impl Type {
    // check whether all the TypeId's are valid
    pub fn is_valid(&self) -> bool {
        let next_type_id = self.context.next_type_id;
        self.context.is_valid() &&
        !(self.i_type.iter().any(|x| *x >= next_type_id) ||
          self.o_type.iter().any(|x| *x >= next_type_id))
    }

    // TODO:
    //     - make method to simplify context

    // TODO: this is next, figure out how to represent/implement replacements of
    //     variables, e.g. starting larger than both or collecting maps (old_lhs -> new_lhs)

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
        let self_context = &self.context;
        let other_context = &other.context;

        let mut context = self_context.clone().new_max(other_context.clone());
        let mut self_type_map: BTreeMap<TypeId, TypeId> = BTreeMap::new();
        let mut other_type_map: BTreeMap<TypeId, TypeId> = BTreeMap::new();

        let mut i_type = vec![];
        let mut o_type = vec![];

        other.o_type.iter().zip(self.i_type.clone()).try_for_each(|(o_type, i_type)| {
            let new_type_id = context
                .unify(self_context.clone(),
                       other_context.clone(),
                       &i_type,
                       &o_type)?;
            self_type_map.insert(i_type, new_type_id);
            other_type_map.insert(*o_type, new_type_id);
            Ok(())
        })?;

        match after_zip(other.o_type.clone(), self.i_type.clone()) {
            Ok(other_o_type_remainder) =>
                for o_type in other_o_type_remainder {
                    let new_o_type = context.push(other.context.clone().get(&o_type)?);
                    other_type_map.insert(o_type.clone(), new_o_type);
                    i_type.push(new_o_type.clone());
                },
            Err(self_i_type_remainder) =>
                for i_type in self_i_type_remainder {
                    let new_i_type = context.push(self.context.clone().get(&i_type)?);
                    self_type_map.insert(i_type.clone(), new_i_type);
                    o_type.push(new_i_type.clone());
                },
        }

        Ok(Type {
            context: context,
            i_type: other.i_type.clone().iter()
                .map(move |x| Ok(other_type_map
                     .get(x)
                     .ok_or_else(|| TypeError::ContextGetUnknownTypeId { // TODO: new error
                         context: other.context.clone(),
                         index: *x,
                    })?.clone())).chain(i_type.iter().map(move |x| Ok(*x))).collect::<Result<Vec<TypeId>, TypeError>>()?,
            o_type: self.o_type.clone().iter()
                .map(|x| Ok(self_type_map
                     .get(x)
                     .ok_or_else(|| TypeError::ContextGetUnknownTypeId {
                         context: self.context.clone(),
                         index: *x,
                    })?.clone())).chain(o_type.iter().map(move |x| Ok(*x))).collect::<Result<Vec<TypeId>, TypeError>>()?,
        })
    }
}

impl Restack {
    // TODO: fix locations
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

#[derive(Debug, PartialEq, Error)]
pub enum TypeError {
    #[error("Context::get applied to a TypeId: {index:?}, not in the Context: {context:?}")]
    ContextGetUnknownTypeId {
        context: Context,
        index: TypeId,
    },

    #[error("Instruction::type_of resulted in restack error: {0:?}")]
    InstructionTypeOfRestack(RestackError),

    #[error("Instruction::type_of resulted in an error involving: {instruction:?};\n {error:?}")]
    InstructionTypeOfDetail {
        instruction: Instruction,
        error: Box<Self>,
    },

    #[error("ElemType::unify applied to non-intersecting types: lhs: {lhs:?}; rhs: {rhs:?}")]
    ElemTypeUnifyEmpty {
        lhs: ElemType,
        rhs: ElemType,
        // location: TyUnifyLocation,
    },

    #[error("Instructions::type_of called on an empty Vec of Instruction's")]
    InstructionsTypeOfEmpty,

    #[error("Instructions::type_of resulted in an error on line: {line_no:?};\n {error:?}")]
    InstructionsTypeOfLineNo {
        line_no: usize,
        error: Box<Self>,
    },


    // // should be impossible
    // #[error("StackTy::unify produced an attempt to unify None and None: lhs: {lhs:?}; rhs: {rhs:?}")]
    // StackTyUnifyNone {
    //     lhs: Vec<Ty>,
    //     rhs: Vec<Ty>,
    // },

    // #[error("attempt to unify types of non-contiguous locations: lhs: {0:?}")]
    // SrcRangeError(SrcRangeError),
}

// impl From<SrcRangeError> for TypeError {
//     fn from(error: SrcRangeError) -> Self {
//         Self::SrcRangeError(error)
//     }
// }


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
        let mut instructions = self.instructions.clone();
        let first_instruction = instructions.pop().ok_or_else(|| TypeError::InstructionsTypeOfEmpty)?;
        let mut current_type = first_instruction.type_of(From::from(0))?;
        for (i, instruction) in instructions.iter().enumerate() {
            current_type = current_type.compose(instruction.type_of(From::from(i + 1))?)
                .or_else(|e| Err(TypeError::InstructionsTypeOfLineNo { // TODO: deprecated by Location
                    line_no: i,
                    error: Box::new(e),
                }))?;
        }
        Ok(current_type)
    }
}

