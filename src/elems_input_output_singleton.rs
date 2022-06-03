use crate::elem::Elem;
use crate::elem_type::ElemType;
use crate::an_elem::AnElem;
use crate::stack::Stack;
use crate::types::context::Context;
use crate::types::Type;
use crate::elems_singleton::Singleton;
use crate::elems::{Elems, ElemsPopError};
use crate::elems_input_output::IOElems;
use crate::an_elem_return::Return;

use std::marker::PhantomData;
use std::fmt::Debug;

use generic_array::{GenericArray, ArrayLength};
use typenum::marker_traits::Unsigned;

/// A Singleton Return-ing AnElem of the same type, but always with a
/// multiplicity of one.
#[derive(Clone, Debug)]
pub struct ReturnSingleton<T, N>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
{
    /// Wrapped Singleton
    pub singleton: Singleton<T, N>,

    /// Typed return slot
    pub returning: Return<T>,
}

impl<T, N> IntoIterator for ReturnSingleton<T, N>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
{
    type Item = Elem;
    type IntoIter = <Singleton<T, N> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.singleton.into_iter()
    }
}

impl<T, N> Elems for ReturnSingleton<T, N>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
{
    type Hd = T;
    type N = N;
    type Tl = Singleton<T, N>;

    // fn left(_s: PhantomData<Self>, x: GenericArray<Self::Hd, Self::N>) -> Self { Elems::left(PhantomData::<Singleton<T, N>>, x) }
    // fn right(_s: PhantomData<Self>, x: Self::Tl) -> Self { Elems::left(PhantomData::<Singleton<T, N>>, x) }
    fn or<U, F: Fn(&GenericArray<Self::Hd, Self::N>) -> U, G: Fn(&Self::Tl) -> U>(&self, f: F, g: G) -> U {
        self.singleton.or(f, g)
    }

    fn pop(_x: PhantomData<Self>, stack: &mut Stack) -> Result<Self, ElemsPopError>
    where
        Self: Sized,
    {
        Ok(ReturnSingleton {
            singleton: Elems::pop(PhantomData::<Singleton<T, N>>, stack)?,
            returning: Return::new(),
        })
    }

    fn elem_type(_t: PhantomData<Self>) -> Result<ElemType, ElemsPopError> {
        Elems::elem_type(PhantomData::<Singleton<T, N>>)
    }
}

impl<T, N> IOElems for ReturnSingleton<T, N>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
{
    fn or_return<U, F, G>(&self, f: F, _g: G) -> U
    where
        F: Fn(&GenericArray<Self::Hd, Self::N>, &Return<Self::Hd>) -> U,
        G: Fn(&Self::Tl) -> U,
    {
        f(&self.singleton.array, &self.returning)
    }

    fn returning(&self) -> Option<Elem> {
        self.returning.returned().map(|x| x.to_elem())
    }

    fn type_of(_t: PhantomData<Self>) -> Result<Type, ElemsPopError> {
        let num_inputs = <N as Unsigned>::to_usize();
        let mut context = Context::new();
        let type_id = context.push(ElemType {
            type_set: AnElem::elem_symbol(PhantomData::<T>),
            info: vec![],
        });
        Ok(Type {
            context,
            i_type: (1..num_inputs).into_iter().map(|_| type_id).collect(),
            o_type: vec![type_id],
        })
    }
}

