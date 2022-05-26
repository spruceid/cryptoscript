use crate::an_elem::AnElem;
use crate::elems_singleton::Singleton;
use crate::elems_or::Or;
use crate::elems::Elems;

use std::fmt::Debug;

use generic_array::ArrayLength;

/// Input Elems
pub trait IElems: Elems {}

impl<T, N> IElems for Singleton<T, N>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
{}

impl<T, N, U> IElems for Or<T, N, U>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
    U: IElems<N = N>,
{}

