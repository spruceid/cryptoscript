use crate::stack::Stack;
use crate::elem::Elem;
use crate::elem_type::ElemType;
use crate::an_elem::AnElem;
use crate::elems_singleton::Singleton;
use crate::elems::{Elems, ElemsPopError};

use std::marker::PhantomData;
use std::fmt::{self, Debug, Formatter};
use std::sync::Arc;

use generic_array::{GenericArray, ArrayLength};

/// Either AnElem with type T an multiplicity N or Elems U, i.e.
/// Or is equivalent to Result<Singleton<T, N>, U> with constraints
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Or<T, N, U>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
    U: Elems<N = N>,
{
    /// AnElem with type T and multiplicity N. Equivalent to Singleton unwrapped
    Left(GenericArray<T, N>),
    /// Other Elems
    Right(U),
}

pub enum IterOr<T, N, U>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
    U: Elems<N = N>,
{
    Left(<Singleton<T, N> as IntoIterator>::IntoIter),
    Right(<U as IntoIterator>::IntoIter),
}

impl<T, N, U> Debug for IterOr<T, N, U>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
    U: Elems<N = N>,
    <U as IntoIterator>::IntoIter: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::Left(x) => write!(f, "IterOr::Left({:?})", x),
            Self::Right(x) => write!(f, "IterOr::Right({:?})", x),
        }
    }
}

impl<T, N, U> Iterator for IterOr<T, N, U>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
    U: Elems<N = N>,
{
    type Item = Elem;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Left(x) => x.next(),
            Self::Right(x) => x.next(),
        }
    }
}

impl<T, N, U> IntoIterator for Or<T, N, U>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
    U: Elems<N = N>,
{
    type Item = Elem;
    type IntoIter = IterOr<T, N, U>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Left(array) => IterOr::Left(
                Singleton {
                    array,
                }.into_iter()
            ),
            Self::Right(xs) => IterOr::Right(xs.into_iter()),
        }
    }
}

impl<T, N, U> Elems for Or<T, N, U>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
    U: Elems<N = N>,
{
    type Hd = T;
    type N = N;
    type Tl = U;

    fn or<V, F: Fn(&GenericArray<Self::Hd, Self::N>) -> V, G: Fn(&Self::Tl) -> V>(&self, f: F, g: G) -> V {
        match self {
            Self::Left(x) => f(x),
            Self::Right(x) => g(x),
        }
    }

    fn pop(_x: PhantomData<Self>, stack: &mut Stack) -> Result<Self, ElemsPopError>
    where
        Self: Sized,
    {
        match <Singleton<T, N> as Elems>::pop(PhantomData, stack) {
            Ok(Singleton { array }) => Ok(Self::Left(array)),
            Err(hd_error) => {
                Elems::pop(PhantomData::<U>, stack)
                    .map(|x| Self::Right(x))
                    .map_err(|tl_errors| {
                        ElemsPopError::Pop {
                            hd_error: Arc::new(hd_error),
                            tl_errors: Arc::new(tl_errors),
                        }
                    })
            },
        }
    }

    // TODO: add info
    fn elem_type(_t: PhantomData<Self>) -> Result<ElemType, ElemsPopError> {
        let elem_type_hd = ElemType {
            type_set: AnElem::elem_symbol(PhantomData::<T>),
            info: vec![],
        };
        let mut elem_type_tl = Elems::elem_type(PhantomData::<U>)?;
        Ok(elem_type_hd.union(&mut elem_type_tl))
    }
}

