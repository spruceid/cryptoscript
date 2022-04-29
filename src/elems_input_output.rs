use crate::elem::Elem;
use crate::an_elem_return::Return;
use crate::types::Type;
use crate::elems::{Elems, ElemsPopError};

use std::marker::PhantomData;

use generic_array::GenericArray;

/// A set of optionally-returned AnElem's with multiplicities
pub trait IOElems: Elems {
    /// Unpack either of the Left/Right options:
    /// - The Left case is a GenericArray of Self::N copies of Self::Hd,
    ///     returning Self::Hd
    /// - The Right case is Self::Tl
    fn or_return<T, F, G>(&self, f: F, g: G) -> T
        where
            F: Fn(&GenericArray<Self::Hd, Self::N>, &Return<Self::Hd>) -> T,
            G: Fn(&Self::Tl) -> T;

    // TODO: rename to 'returned' to match Return<T>
    /// The returned Elem, if any has been returned
    fn returning(&self) -> Option<Elem>;

    /// The type of the set of optionally-returned AnElem's with multiplicities
    fn type_of(t: PhantomData<Self>) -> Result<Type, ElemsPopError>;
}

