use crate::elem_type::{ElemType, ElemTypeError, StackType};
use crate::types::type_id::TypeId;
use crate::types::type_id::map::{TypeIdMap, TypeIdMapError};
use crate::types::Type;

use std::cmp;
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::fmt;
use std::sync::Arc;

use thiserror::Error;

/// A context defining associations between TypeId's and ElemType's
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Context {
    context: BTreeMap<TypeId, ElemType>,

    /// TODO: make read-only
    pub next_type_id: TypeId,
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
                &format!("∀ ({t_i} ∊ {xs}),", t_i = i.debug(), xs = xs)
              }))
    }
}

#[cfg(test)]
mod context_display_tests {
    use super::*;
    use enumset::EnumSet;

    #[test]
    fn test_empty() {
        let big_type_id = TypeId::new(2^32);
        let context = Context::new().offset(big_type_id);
        assert_eq!("", format!("{}", context));
    }

    #[test]
    fn test_singleton() {
        for elem_symbol in EnumSet::all().iter() {
            let elem_type = ElemType {
                type_set: EnumSet::only(elem_symbol),
                info: vec![],
            };
            let mut context = Context::new();
            context.push(elem_type.clone());
            assert_eq!(format!("\n∀ (t0 ∊ {}),", elem_type), format!("{}", context));
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    /// New empty context with next_type_id = 0.
    pub fn new() -> Self {
        Context {
            context: BTreeMap::new(),
            next_type_id: TypeId::new(0),
        }
    }

    /// Is self.context valid with respect to self.next_type_id?
    pub fn is_valid(&self) -> bool {
        !self.context.keys().any(|x| *x >= self.next_type_id)
    }

    /// The size of self.context
    pub fn size(&self) -> usize {
        self.context.len()
    }

    /// Push a new ElemType onto the Context, returning its TypeId
    pub fn push(&mut self, elem_type: ElemType) -> TypeId {
        let push_id = self.next_type_id;
        self.context.insert(push_id, elem_type);
        self.next_type_id = push_id.offset(TypeId::new(1));
        push_id
    }

    /// Normalize the naming of TypeId's along the given basis vector, returning a TypeIdMap with
    /// the new associations.
    ///
    /// Note: NormalizeOnInvalidBasis is possible iff a `TypeId` in (basis) is repeated
    /// or missing from (self)
    pub fn normalize_on(&self, basis: Vec<TypeId>) -> Result<(Self, TypeIdMap), ContextError> {
        let mut source = self.clone();
        let mut result = Self::new();
        let mut type_map = TypeIdMap::new();
        for &type_id in &basis {
            match source.context.remove(&type_id) {
                None => Err(ContextError::NormalizeOnInvalidBasis {
                    type_id,
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

    /// Offset all TypeId's
    pub fn offset(&self, offset: TypeId) -> Self {
        Context {
            context: self.context.iter().map(|(k, x)| (k.offset(offset), x.clone())).collect(),
            next_type_id: self.next_type_id.offset(offset),
        }
    }

    /// Update a TypeId, fails if:
    /// - The "from" destination TypeId does not exist in self.context
    /// - The "to" destination TypeId already exists in self.context
    pub fn update_type_id(&mut self, from: TypeId, to: TypeId) -> Result<(), ContextError> {
        if self.context.contains_key(&from) {
            Ok(())
        } else {
            Err(ContextError::UpdateTypeIdFromMissing {
                from,
                to,
                context: self.clone(),
            })
        }?;
        if self.context.contains_key(&to) {
            Err(ContextError::UpdateTypeIdToPresent {
                from,
                to,
                context: self.clone(),
            })
        } else {
            Ok(())
        }?;
        self.context = self.context.iter().map(|(k, x)| (k.update_type_id(from, to), x.clone())).collect();
        self.next_type_id = cmp::max(self.next_type_id, to);
        Ok(())
    }

    /// Disjoint union of two Context's: fails if not disjoint
    pub fn disjoint_union(&mut self, other: Self) -> Result<(), ContextError> {
        for (&type_id, elem_type) in other.context.iter() {
            match self.context.insert(type_id, elem_type.clone()) {
                None => {
                    Ok(())
                },
                Some(conflicting_elem_type) => Err(ContextError::DisjointUnion {
                    type_id,
                    elem_type: elem_type.clone(),
                    conflicting_elem_type,
                    lhs: self.clone(),
                    rhs: other.clone(),
                }),
            }?
        }
        self.next_type_id = cmp::max(self.next_type_id, other.next_type_id);
        Ok(())
    }

    /// Get the ElemType associated with the given TypeId
    pub fn get(&mut self, index: &TypeId, error: &dyn Fn() -> ContextError) -> Result<ElemType, ContextError> {
        Ok(self.context.get(index).ok_or_else(|| ContextError::GetUnknownTypeId {
            context: self.clone(),
            index: *index,
            error: Arc::new(error()),
        })?.clone())
    }

    /// Unify the types of two TypeId's into the RHS,
    /// removing the LHS
    pub fn unify(&mut self, xi: TypeId, yi: TypeId) -> Result<(), ContextError> {
        let x_type = self.context.remove(&xi).ok_or_else(|| ContextError::Unify {
            xs: self.clone(),
            xi,
            yi,
            is_lhs: true,
        })?;
        let mut y_type = self.context.remove(&yi).ok_or_else(|| ContextError::Unify {
            xs: self.clone(),
            xi,
            yi,
            is_lhs: false,
        })?;
        let xy_type = x_type.unify(&mut y_type).map_err(|error| ContextError::UnifyElemType {
            xs: self.clone(),
            xi,
            yi,
            error,
        })?;
        self.context.insert(yi, xy_type);
        Ok(())
    }

    /// Unify the given ElemType into a particular TypeId in self.context
    pub fn unify_elem_type(&mut self, xi: TypeId, elem_type: ElemType) -> Result<(), ContextError> {
        let yi = self.push(elem_type);
        self.unify(xi, yi)
    }

    /// Maximum possible TypeId, not maximum present
    pub fn max_type_id(&self) -> Result<TypeId, ContextError> {
        self.next_type_id.previous().ok_or_else(|| ContextError::MaxTypeId(self.clone()))
        // let type_id = self.next_type_id.type_id;
        // if type_id == 0 {
        //     Err(ContextError::MaxTypeId(self.clone()))
        // } else {
        //     Ok(TypeId {
        //         type_id: type_id - 1,
        //     })
        // }
    }
}

/// Context trait errors
#[derive(Clone, Debug, PartialEq, Error)]
pub enum ContextError {
    /// "Context::get applied to a TypeId: \n{index:?}\n, not in the Context: \n
    /// {context:?}\n, error: \n{error:?}\n"
    #[error("Context::get applied to a TypeId: \n{index:?}\n, not in the Context: \n{context:?}\n, error: \n{error:?}\n")]
    GetUnknownTypeId {
        /// Given Context
        context: Context,

        /// TypeId not found
        index: TypeId,

        /// Associated error
        error: Arc<Self>,
    },

    /// "Context::disjoint_union applied to lhs: \n{lhs:?}\n, and rhs: \n{rhs:?}\n,
    /// with type_id: \n{type_id:?}\n, and elem_type: \n{elem_type:?}\n,
    /// conflicted with lhs entry conflicting_elem_type: {conflicting_elem_type:?\n}\n"
    #[error("Context::disjoint_union applied to lhs: \n{lhs:?}\n, and rhs: \n{rhs:?}\n, /
            with type_id: \n{type_id:?}\n, and elem_type: \n{elem_type:?}\n, conflicted /
            with lhs entry conflicting_elem_type: {conflicting_elem_type:?\n}\n")]
    DisjointUnion {
        /// Conflicting TypeId
        type_id: TypeId,

        /// LHS conflicting ElemType
        elem_type: ElemType,

        /// RHS conflicting ElemType
        conflicting_elem_type: ElemType,

        /// RHS Context
        lhs: Context,

        /// RHS Context
        rhs: Context,
    },

    /// "Context::normalize_on applied to invalid basis: type_id: \n
    /// {type_id:?}\n, context: \n{context:?}\n, basis: \n{basis:?}\n"
    #[error("Context::normalize_on applied to invalid basis: type_id: \n{type_id:?}\n, context: \n{context:?}\n, basis: \n{basis:?}\n")]
    NormalizeOnInvalidBasis {
        /// TypeId invalid for given context, basis
        type_id: TypeId,

        /// Given Context
        context: Context,

        /// Basis of TypeId's to normalize onto: attempts to sort by this basis
        basis: Vec<TypeId>,
    },

    /// "Context::update_type_id called on missing 'from: TypeId':\n from: \n
    /// {from:?}\n to: {to:?}\n context: {context:?}"
    #[error("Context::update_type_id called on missing 'from: TypeId':\n from: \n{from:?}\n to: {to:?}\n context: {context:?}")]
    UpdateTypeIdFromMissing {
        /// Updating TypeId from
        from: TypeId,

        /// Updating TypeId to
        to: TypeId,

        /// Given Context
        context: Context,
    },

    /// "Context::update_type_id called on already-present 'to: TypeId':\n from: \n
    /// {from:?}\n\n to: \n{to:?}\n context: \n{context:?}\n"
    #[error("Context::update_type_id called on already-present 'to: TypeId':\n from: \n{from:?}\n\n to: \n{to:?}\n context: \n{context:?}\n")]
    UpdateTypeIdToPresent {
        /// Updating TypeId from
        from: TypeId,

        /// Updating TypeId to
        to: TypeId,

        /// Given Context
        context: Context,
    },

    /// "Context::unify failed:\n xs: \n{xs:?}\n xi: \n{xi:?}\n yi: \n{yi:?}\n
    /// is_lhs: \n{is_lhs:?}\n"
    #[error("Context::unify failed:\n xs: \n{xs:?}\n xi: \n{xi:?}\n yi: \n{yi:?}\n is_lhs: \n{is_lhs:?}\n")]
    Unify {
        /// Given Context
        xs: Context,

        /// RHS
        xi: TypeId,

        /// LHS
        yi: TypeId,

        /// Is it on the LHS?
        is_lhs: bool,
    },

    /// "Context::unify failed to unify ElemType's:\n\nxs:\n{xs}\n\nxi:\n{xi}\n\n
    /// yi:\n{yi}\n\nelem_error:\n{error}\n"
    #[error("Context::unify failed to unify ElemType's:\n\nxs:\n{xs}\n\nxi:\n{xi}\n\nyi:\n{yi}\n\nelem_error:\n{error}\n")]
    UnifyElemType {
        /// Given Context
        xs: Context,

        /// RHS TypeId
        xi: TypeId,

        /// LHS TypeId
        yi: TypeId,

        /// ElemTypeError
        error: ElemTypeError,
    },

    /// "Type::specialize_to_input_stack failed to resolve ElemType's:\ntype_of:\n
    /// {type_of}\n\nstack_type:\n{stack_type}"
    #[error("Type::specialize_to_input_stack failed to resolve ElemType's:\ntype_of:\n{type_of}\n\nstack_type:\n{stack_type}")]
    SpecializeToInputStack {
        /// The type being specialized
        type_of: Type,

        /// Stack type attempted to specialize to
        stack_type: StackType,
    },

    /// "Context::normalize_on building TypeIdMap failed: \n{0:?}\n"
    #[error("Context::normalize_on building TypeIdMap failed: \n{0:?}\n")]
    TypeIdMapError(TypeIdMapError),

    /// "Context::max_type_id: next_type_id == 0: \n{0:?}\n"
    #[error("Context::max_type_id: next_type_id == 0: \n{0:?}\n")]
    MaxTypeId(Context),
}

impl From<TypeIdMapError> for ContextError {
    fn from(error: TypeIdMapError) -> Self {
        Self::TypeIdMapError(error)
    }
}

