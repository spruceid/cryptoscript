use crate::stack::Stack;
use crate::elem::Elem;
use crate::elem_type::ElemType;
use crate::an_elem::AnElem;
use crate::elems::{Elems, ElemsPopError};

use std::fmt::Debug;
use std::marker::PhantomData;

use generic_array::{GenericArray, GenericArrayIter, ArrayLength};
use typenum::marker_traits::Unsigned;

// TODO: rename
/// AnElem with type T and multiplicity N
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Singleton<T, N>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
{
    /// An array of AnElem with multiplicity N
    pub array: GenericArray<T, N>,
}

impl<T, N> IntoIterator for Singleton<T, N>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
{
    type Item = Elem;
    type IntoIter = std::iter::Map<GenericArrayIter<T, N>, fn(T) -> Elem>;

    fn into_iter(self) -> Self::IntoIter {
        self.array.into_iter().map(AnElem::to_elem)
    }
}

impl<T, N> Elems for Singleton<T, N>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
{
    type Hd = T;
    type N = N;
    type Tl = Singleton<T, N>;

    // fn left(_s: PhantomData<Self>, x: GenericArray<Self::Hd, Self::N>) -> Self { Singleton { t: x, } }
    // fn right(_s: PhantomData<Self>, x: Self::Tl) -> Self { x }
    fn or<U, F: Fn(&GenericArray<Self::Hd, Self::N>) -> U, G: Fn(&Self::Tl) -> U>(&self, f: F, _g: G) -> U {
        f(&self.array)
    }

    fn pop(_x: PhantomData<Self>, stack: &mut Stack) -> Result<Self, ElemsPopError>
    where
        Self: Sized,
    {
        let vec = (0..<N as Unsigned>::to_usize()).map(|_array_ix| {
            stack
                .pop_elem(PhantomData::<T>)
                .map_err(|e| ElemsPopError::PopSingleton {
                    elem_symbol: AnElem::elem_symbol(PhantomData::<T>),
                    error: e,
                })
        }).collect::<Result<Vec<T>, ElemsPopError>>()?;
        let array = GenericArray::from_exact_iter(vec.clone()).ok_or_else(|| {
            ElemsPopError::GenericArray {
                elem_set: AnElem::elem_symbol(PhantomData::<T>),
                vec: vec.into_iter().map(|x| x.to_elem()).collect(),
                size: <N as Unsigned>::to_usize(),
            }
        })?;
        Ok(Singleton {
            array: array,
        })
    }

    // TODO: add info
    fn elem_type(_t: PhantomData<Self>) -> Result<ElemType, ElemsPopError> {
        Ok(ElemType {
            type_set: AnElem::elem_symbol(PhantomData::<T>),
            info: vec![],
        })
    }
}

