pub(crate) mod empty;

pub(crate) mod type_id;
use type_id::TypeId;
use type_id::map::TypeIdMapError;

pub(crate) mod context;
use context::{Context, ContextError};

use crate::restack::{Restack, RestackError};
use crate::location::LineNo;
use crate::elem_type::{ElemType, StackType};

use std::fmt::{Display, Formatter};
use std::fmt;

use thiserror::Error;

// typing:
// - unification
//   + inference
//   + checking against inferred or other type (this + inference = bidirecitonal)
// - two categories of tests:
//   + property tests for typing methods themselves
//   + test that a function having a particular type -> it runs w/o type errors on such inputs

// TODO: make fields private
/// Type of a series of instructions
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Type {
    /// TypeId Context, assigning ElemType's to each TypeId
    pub context: Context,

    /// Input type stack (all TypeId's must be in the Context)
    pub i_type: Vec<TypeId>,

    /// Output type stack (all TypeId's must be in the Context)
    pub o_type: Vec<TypeId>,
}

impl Type {
    /// Identity Type
    pub fn id() -> Self {
        Type {
            context: Context::new(),
            i_type: vec![],
            o_type: vec![],
        }
    }

    /// The next TypeId, guaranteed to not be present in the Context
    pub fn next_type_id(&self) -> TypeId {
        self.context.next_type_id
    }

    /// check whether all the TypeId's are valid
    pub fn is_valid(&self) -> bool {
        let next_type_id = self.next_type_id();
        self.context.is_valid() &&
        !(self.i_type.iter().any(|x| *x >= next_type_id) ||
          self.o_type.iter().any(|x| *x >= next_type_id))
    }

    /// Equivalent to running update_type_id w/ offset from largest to smallest
    /// existing TypeId
    pub fn offset(&self, offset: TypeId) -> Self {
        Type {
            context: self.context.offset(offset),
            i_type: self.i_type.iter().map(|x| x.offset(offset)).collect(),
            o_type: self.o_type.iter().map(|x| x.offset(offset)).collect(),
        }
    }

    /// Update a TypeId, failing if "from" isn't present or "to" already is
    pub fn update_type_id(&mut self, from: TypeId, to: TypeId) -> Result<(), TypeError> {
        self.context.update_type_id(from, to).map_err(|e| TypeError::UpdateTypeId(e))?;
        self.i_type = self.i_type.iter().map(|x| x.update_type_id(from, to)).collect();
        self.o_type = self.o_type.iter().map(|x| x.update_type_id(from, to)).collect();
        Ok(())
    }

    /// Normalize self.context on self.i_type as a basis
    pub fn normalize(&self) -> Result<Self, TypeError> {
        let mut basis = self.i_type.clone();
        basis.append(&mut self.o_type.clone());
        basis.dedup();
        let (new_context, type_map) = self.context.normalize_on(basis).map_err(|e| TypeError::NormalizeContextError(e))?;
        Ok(Type {
            context: new_context,
            i_type: type_map.run(self.i_type.clone()).map_err(|e| TypeError::TypeIdMapError(e))?,
            o_type: type_map.run(self.o_type.clone()).map_err(|e| TypeError::TypeIdMapError(e))?,
        })
    }

    /// Specialize self to the given StackType, or fail if it's not a valid
    /// specialization.
    ///
    /// Returns the output stack
    pub fn specialize_to_input_stack(&mut self, stack_type: StackType) -> Result<StackType, TypeError> {
        if self.i_type.len() <= stack_type.len() {
            let mut stack_type_iter = stack_type.clone().into_iter();
            for (type_id, elem_type) in self.i_type.clone().into_iter().zip(&mut stack_type_iter) {
                // TODO: elimate copy?
                let elem_type_copy = elem_type.clone();
                self.context.unify_elem_type(type_id, elem_type).map_err(|e| TypeError::Specialization {
                    type_id: type_id,
                    elem_type: elem_type_copy,
                    context: self.context.clone(),
                    error: e,
                })?
            }
            for elem_type in stack_type_iter {
                let type_id = self.context.push(elem_type);
                self.i_type.push(type_id);
            }
            // Ok(())

            Ok(StackType {
                types: self.o_type.clone().into_iter().map(|type_id| {
                    self.context.clone().get(&type_id, &|| ContextError::SpecializeToInputStack {
                        type_of: self.clone(),
                        stack_type: stack_type.clone(),
                    })
                }).collect::<Result<Vec<ElemType>, ContextError>>()
                       .map_err(|e| TypeError::SpecializeToInputStackContextError(e))?,
            })

        } else {
            Err(TypeError::SpecializeToInputStack {
                type_of: self.clone(),
                stack_type: stack_type.clone(),
            })
        }
    }

    /// Unify two Type's by producing the type of their composition.
    ///
    /// ```
    /// f : self
    /// g : other
    /// self.compose(other) : (f ++ g).type_of()
    ///
    /// input ->
    /// other.i_type
    /// other.o_type
    /// self.i_type
    /// self.o_type
    /// -> output
    /// ```
    ///
    /// 1. iterate through (zip(self.o_type, other.i_type)) and unify the pairs into a new context
    /// 2. collect the remainder and add them to the context
    /// 3. add the remainder to (self.i_type, other.o_type), with replaced variables
    pub fn compose(&self, other: Self) -> Result<Self, TypeError> {
        println!("");
        println!("composing:\n{0}\n\nAND\n{1}\n", self, other);

        let mut context = self.context.clone();
        // println!("context: {}", context);
        // println!("context.next_type_id: {:?}", context.next_type_id.type_id);

        let offset_other = other.offset(self.next_type_id());
        // println!("offset_other: {}", offset_other);

        context.disjoint_union(offset_other.context.clone())
            .map_err(|e| TypeError::ComposeContextError(e))?;
        // println!("context union: {}", context);

        let mut mut_offset_other = offset_other.clone();
        let mut zip_len = 0;
        let other_o_type = offset_other.o_type.iter().clone();
        let self_i_type = self.i_type.iter().clone();
        other_o_type.zip(self_i_type).try_for_each(|(&o_type, &i_type)| {
            zip_len += 1;
            context
                .unify(o_type, i_type)
                .map_err(|e| TypeError::ComposeContextError(e))?;
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

    /// Prepend inputs to self.i_type
    pub fn prepend_inputs(&mut self, num_copies: usize, elem_type: ElemType) -> () {
        if 0 < num_copies {
            let type_id = self.context.push(elem_type);
            self.i_type = (1..num_copies).into_iter()
                .map(|_| type_id)
                .chain(self.i_type.clone().into_iter())
                .collect()
        }
    }

    /// Append inputs to self.i_type
    pub fn append_inputs<T>(&mut self, elem_types: T) -> ()
    where
        T: IntoIterator<Item = ElemType>,
    {
        for elem_type in elem_types {
            let type_id = self.context.push(elem_type);
            self.i_type.push(type_id)
        }
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
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        // TODO: fix normalize
        // let self_normalized = self.normalize().map_err(|_| fmt::Error)?;
        let self_normalized = self;
        write!(f,
               "{context}\n[{i_type}] ->\n[{o_type}]",
               context = self_normalized.context,
               i_type = self_normalized.i_type.iter().fold(String::new(), |memo, x| {
                   let x_str = x.debug();
                   if memo == "" {
                       x_str
                   } else {
                       memo + ", " + &x_str
                   }}),
               o_type = self_normalized.o_type.iter().fold(String::new(), |memo, x| {
                   let x_str = x.debug();
                   if memo == "" {
                       x_str
                   } else {
                       memo + ", " + &x_str
                   }}))
    }
}

#[cfg(test)]
mod type_display_tests {
    use super::*;
    use enumset::EnumSet;

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

/// Type trait errors
#[derive(Clone, Debug, PartialEq, Error)]
pub enum TypeError {
    /// "Specialization error:\ntype_id:\n{type_id}\n\nelem_type:\n{elem_type}\n\ncontext:\n{context}\n\nerror:\n{error}"
    #[error("Specialization error:\ntype_id:\n{type_id}\n\nelem_type:\n{elem_type}\n\ncontext:\n{context}\n\nerror:\n{error}")]
    Specialization {
        /// ElemType with this TypeId is invalid specialization of Context
        type_id: TypeId,

        /// ElemType for type_id is invalid specialization of Context
        elem_type: ElemType,

        /// Context not compatible with TypeId, ElemType pair
        context: Context,

        /// ContextError
        error: ContextError,
    },

    /// "NormalizeContextError\n{0}"
    #[error("NormalizeContextError\n{0}")]
    NormalizeContextError(ContextError),

    /// "ComposeContextError\n{0}"
    #[error("ComposeContextError\n{0}")]
    ComposeContextError(ContextError),

    /// "TypeError::update_type_id failed when updating the Context:\n{0}"
    #[error("TypeError::update_type_id failed when updating the Context:\n{0}")]
    UpdateTypeId(ContextError),

    /// "TypeError::compose disjoint_union\n{0}"
    #[error("TypeError::compose disjoint_union\n{0}")]
    ComposeDisjointUnion(ContextError),

    /// "Type::normalize applying TypeIdMap failed:\n{0}"
    #[error("Type::normalize applying TypeIdMap failed:\n{0}")]
    TypeIdMapError(TypeIdMapError),

    /// "Type::specialize_to_input_stack ContextError:\n{0}"
    #[error("Type::specialize_to_input_stack ContextError:\n{0}")]
    SpecializeToInputStackContextError(ContextError),

    // TODO: use StackType and Display instead of Vec
    /// "Type::specialize_to_input_stack: stack_type shorter than expected:\n{type_of}\n{stack_type}"
    #[error("Type::specialize_to_input_stack: stack_type shorter than expected:\n{type_of}\n{stack_type}")]
    SpecializeToInputStack {
        /// Type too long for stack_type
        type_of: Type,

        /// Shorter than expected StackType
        stack_type: StackType,
    },
}


impl Restack {
    /// Calculate the Type of a Restack instruction
    ///
    /// In short, the input stack is [x_1, x_2, .. x_restack_depth]
    /// and the output stack is self.restack(input_stack)
    // TODO: fix locations: out locations are mislabeled as in locations
    pub fn type_of(&self, line_no: LineNo) -> Result<Type, RestackError> {
        let mut context = Context::new();
        let mut restack_type: Vec<TypeId> = (0..self.restack_depth)
            .map(|x| context.push(ElemType::any(vec![line_no.in_at(x)])))
            .collect();
        let i_type = restack_type.clone();
        self.run(&mut restack_type)?;
        Ok(Type {
            context: context,
            i_type: i_type,
            o_type: restack_type,
        })
    }
}

