use crate::elem::Elem;
use crate::elem_type::ElemType;
use crate::an_elem::AnElem;
use crate::stack::Stack;
use crate::types::Type;
use crate::elems_or::Or;
use crate::elems::{Elems, ElemsPopError};
use crate::elems_input_output::IOElems;
use crate::an_elem_return::Return;

use std::marker::PhantomData;
use std::fmt::Debug;
use std::sync::Arc;

use generic_array::{GenericArray, ArrayLength};

/// A version of Or where the Left side is equivalent to ReturnSingleton<T, N>,
/// i.e. Or<ReturnSingleton<T, N>, U> with appropriate trait constraints to
/// ensure that exactly one typed value is returned.
#[derive(Clone, Debug)]
pub enum ReturnOr<T, N, U>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
    U: Elems<N = N>,
{
    /// ReturnSingleton<T, N>
    Left {
        /// N copies of AnElem type T
        array: GenericArray<T, N>,

        /// Returning a single copy of AnElem type T
        returning: Return<T>,
    },

    /// The Right or continuation variant. See Or.
    Right(U),
}

impl<T, N, U> IntoIterator for ReturnOr<T, N, U>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
    U: Elems<N = N>,
{
    type Item = Elem;
    type IntoIter = <Or<T, N, U> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Left { array, .. } => Or::<T, N, U>::Left(array).into_iter(),
            Self::Right(xs) => Or::Right(xs).into_iter(),
        }
    }
}

impl<T, N, U> Elems for ReturnOr<T, N, U>
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
            Self::Left { array, .. } => f(array),
            Self::Right(x) => g(x),
        }
    }

    fn pop(_x: PhantomData<Self>, stack: &mut Stack) -> Result<Self, ElemsPopError>
    where
        Self: Sized,
    {
        <Or<T, N, U> as Elems>::pop(PhantomData, stack)
            .map(|x| {
                match x {
                    Or::Left(array) => Self::Left {
                        array: array,
                        returning: Return::new(),
                    },
                    Or::Right(y) => Self::Right(y),
                }
            })
    }

    fn elem_type(_t: PhantomData<Self>) -> Result<ElemType, ElemsPopError> {
        Elems::elem_type(PhantomData::<Or<T, N, U>>)
    }
}

impl<T, N, U> IOElems for ReturnOr<T, N, U>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
    U: IOElems<N = N>
{
    fn or_return<V, F, G>(&self, f: F, g: G) -> V
    where
        F: Fn(&GenericArray<Self::Hd, Self::N>, &Return<Self::Hd>) -> V,
        G: Fn(&Self::Tl) -> V,
    {
        match self {
            Self::Left { array, returning } => {
                f(array, returning)
            },
            Self::Right(x) => g(x),
        }
    }

    fn returning(&self) -> Option<Elem> {
        match self {
            Self::Left { returning, .. } => {
                returning.returned().map(|x| x.to_elem())
            },
            Self::Right(x) => x.returning(),
        }
    }

    // TODO: add error info
    fn type_of(_t: PhantomData<Self>) -> Result<Type, ElemsPopError> {
        let mut type_tl = IOElems::type_of(PhantomData::<U>)
            .map_err(|e| ElemsPopError::ReturnOrTl(Arc::new(e)))?;
        let last_type_id = type_tl.context.max_type_id()
            .map_err(|e| ElemsPopError::ReturnOrContextError(e))?;
        let next_type_id = type_tl.context.push(ElemType {
            type_set: AnElem::elem_symbol(PhantomData::<T>),
            info: vec![],
        });
        type_tl.context.unify(last_type_id, next_type_id)
            .map_err(|e| ElemsPopError::ReturnOrContextError(e))?;
        Ok(type_tl)
    }
}





