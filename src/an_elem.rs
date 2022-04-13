use crate::elem::{Elem, ElemSymbol};

use thiserror::Error;

use std::fmt::Debug;
use std::marker::PhantomData;

use enumset::EnumSet;
use serde_json::{Map, Number, Value};

/// Valid Elem(ent) types
///
/// TODO: make closed
pub trait AnElem: Clone + Debug + PartialEq {
    // TODO: rename?

    // fn elem_symbol(t: PhantomData<Self>) -> ElemType;
    /// The ElemSymbol's associated with the Elem's that can form this type
    fn elem_symbol(t: PhantomData<Self>) -> EnumSet<ElemSymbol>;

    /// Convert the Self to Elem by using one of Elem's constructors
    fn to_elem(self) -> Elem;

    /// Convert the given Elem to Self through pattern-matching
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


/// AnElem::from_elem errors
#[derive(Clone, Debug, Error)]
pub enum AnElemError {
    /// AnElem::from_elem: element popped from the Stack wasn't the expected type
    #[error("AnElem::from_elem: element popped from the stack\n\n{found}\n\nwasn't the expected type:\n{expected:?}")]
    UnexpectedElemType {
        /// ElemSymbol's expected to be popped from the Stack
        expected: EnumSet<ElemSymbol>,

        /// Elem popped from the Stack
        found: Elem,
    },

    /// Converting Elem to Or failed
    #[error("<Or<_, _> as AnElem>::from_elem: {e_hd:?}\n{e_tl:?}")]
    PopOr {
        /// x in Or<x, y> 
        e_hd: Box<Self>,

        /// y in Or<x, y> 
        e_tl: Box<Self>,
    },
}
