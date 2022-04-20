use crate::restack::RestackError;
use crate::elem::{Elem, ElemSymbol};
use crate::elem_type::{ElemType, ElemTypeError, StackType};
use crate::an_elem::AnElem;
use crate::stack::{Stack, StackError};
use crate::types::ContextError;

use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;

use enumset::EnumSet;
use generic_array::{GenericArray, ArrayLength};
use thiserror::Error;

// TODO:
// - random type -> ~random inhabitant of the type
// - random typed program?

/// Errors thrown by Elems::pop
#[derive(Clone, Debug, Error)]
pub enum ElemsPopError {
    /// "Elems::pop singleton: tried to pop an Elem that was not found:\nelem_symbol:\n{elem_symbol:?}\n\n{error}"
    #[error("Elems::pop singleton: tried to pop an Elem that was not found:\nelem_symbol:\n{elem_symbol:?}\n\n{error}")]
    PopSingleton {
        /// Expected type set
        elem_symbol: EnumSet<ElemSymbol>,
        /// Extended StackError
        error: StackError,
    },

    /// "Elems::pop: tried to pop a set of Elem's that were not found:\n{hd_error}\n\n{tl_errors}"
    #[error("Elems::pop: tried to pop a set of Elem's that were not found:\n{hd_error}\n\n{tl_errors}")]
    Pop {
        /// Self::Hd pop error
        hd_error: Arc<Self>,
        /// Self::Tl pop error
        tl_errors: Arc<Self>,
    },

    /// "Elems::pop: generic_array internal error\n\nelem_set:\n{elem_set:?}\n\nvec:\n{vec:?}\n\nsize:\n{size}"
    // TODO: add detail
    #[error("Elems::pop: generic_array internal error\n\nelem_set:\n{elem_set:?}\n\nvec:\n{vec:?}\n\nsize:\n{size}")]
    GenericArray {
        /// Expected type set
        elem_set: EnumSet<ElemSymbol>,
        /// Found Elem's
        vec: Vec<Elem>,
        /// Expected size
        size: usize,
    },

    /// "IsList::pop (Cons, Hd): tried to pop a set of Elem's that were not found:\nstack_type:\n{stack_type}\n\nelem_set:\n{elem_set}\n\nstack_type:\n{stack_type_of}\n\nerror:\n{error}"
    #[error("IsList::pop (Cons, Hd): tried to pop a set of Elem's that were not found:\nstack_type:\n{stack_type}\n\nelem_set:\n{elem_set}\n\nstack_type:\n{stack_type_of}\n\nerror:\n{error}")]
    IsListHd {
        /// Stack found
        stack_type: StackType,
        /// Expected type
        elem_set: ElemType,
        /// Stack type found
        stack_type_of: StackType,
        /// Extended error
        error: Arc<Self>,
    },

    /// "IsList::pop (Cons, Tl): tried to pop a set of Elem's that were not found:\nstack_type:\n{stack_type}\n\nstack_type_of:\n{stack_type_of}\n\nerror:\n{error}"
    #[error("IsList::pop (Cons, Tl): tried to pop a set of Elem's that were not found:\nstack_type:\n{stack_type}\n\nstack_type_of:\n{stack_type_of}\n\nerror:\n{error}")]
    IsListTl {
        /// Stack found
        stack_type: StackType,
        /// Expected type
        stack_type_of: StackType,
        /// Extended error
        error: Arc<Self>,
    },

    /// "Instr::run: ElemTypeError:\n{0}"
    #[error("Instr::run: ElemTypeError:\n{0}")]
    RestackError(RestackError),

    /// "Elems::elem_type (Or): Set includes repeated type:\n{0}"
    #[error("Elems::elem_type (Or): Set includes repeated type:\n{0}")]
    ElemTypeError(ElemTypeError),

    /// "<ReturnOr as IOElems>::type_of(): ContextError when adding Tl type: {0:?}"
    #[error("<ReturnOr as IOElems>::type_of(): ContextError when adding Tl type: {0:?}")]
    ReturnOrTl(Arc<ElemsPopError>),

    /// "<ReturnOr as IOElems>::type_of(): ContextError when adding type:\n{0}"
    #[error("<ReturnOr as IOElems>::type_of(): ContextError when adding type:\n{0}")]
    ReturnOrContextError(ContextError),
}

/// A set of Elem's with multiplicities, given by Self::N
pub trait Elems: Clone + Debug + IntoIterator<Item = Elem> {
    /// Head Elem
    type Hd: AnElem;
    /// Multiplicity of the head Elem
    type N: ArrayLength<Self::Hd>;
    /// Tail Elems, or Nil
    type Tl: Elems<N = Self::N>;

    // fn left(s: PhantomData<Self>, x: GenericArray<Self::Hd, Self::N>) -> Self;
    // fn right(s: PhantomData<Self>, x: Self::Tl) -> Self;

    /// Unpack Self given handlers for Self::Hd and Self::Tl
    fn or<T, F: Fn(&GenericArray<Self::Hd, Self::N>) -> T, G: Fn(&Self::Tl) -> T>(&self, f: F, g: G) -> T;

    /// Pop Self from a mutable Stack
    fn pop(_x: PhantomData<Self>, stack: &mut Stack) -> Result<Self, ElemsPopError>
    where
        Self: Sized;

    /// Convert to an ElemType
    fn elem_type(t: PhantomData<Self>) -> Result<ElemType, ElemsPopError>;
}

