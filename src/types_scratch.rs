use crate::elem::{Elem, ElemSymbol};
use crate::stack::{Stack, StackError};
use crate::types::{Empty, AnElem, AnError, Nil};

use std::iter::{FromIterator};
use std::marker::PhantomData;

use enumset::EnumSet;
use generic_array::typenum::{U0, U2};
use generic_array::sequence::GenericSequence;
use generic_array::{GenericArray, GenericArrayIter, ArrayLength};
use typenum::marker_traits::Unsigned;


pub trait Elems: AnElem {
    type Hd: AnElem;
    type Tl: Elems;

    fn is_left(&self) -> bool;
    fn left(s: PhantomData<Self>, x: Self::Hd) -> Self;
    fn right(s: PhantomData<Self>, x: Self::Tl) -> Self;
    fn or<T, F: Fn(Self::Hd) -> T, G: Fn(Self::Tl) -> T>(&self, f: F, g: G) -> T;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Singleton<T: AnElem> {
    t: T,
}

impl<T: AnElem> AnElem for Singleton<T> {
    fn elem_symbol(_t: PhantomData<Self>) -> EnumSet<ElemSymbol> {
        <T as AnElem>::elem_symbol(PhantomData)
    }

    fn to_elem(self) -> Elem {
        self.t.to_elem()
    }

    fn from_elem(_t: PhantomData<Self>, x: Elem) -> Result<Self, StackError> {
        <T as AnElem>::from_elem(PhantomData, x).map(|y| {
            Singleton {
                t: y,
            }
        })
    }
}

impl<T: AnElem> Elems for Singleton<T> {
    type Hd = T;
    type Tl = Singleton<T>;

    fn is_left(&self) -> bool { true }

    fn left(_s: PhantomData<Self>, x: Self::Hd) -> Self {
        Singleton {
            t: x,
        }
    }

    fn right(_s: PhantomData<Self>, x: Self::Tl) -> Self {
        x
    }

    fn or<U, F: Fn(Self::Hd) -> U, G: Fn(Self::Tl) -> U>(&self, f: F, _g: G) -> U {
        f(self.t.clone())
    }
}


#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Or<T: AnElem, U: Elems> {
    Left(T),
    Right(U),
}

impl<T: AnElem, U: Elems> AnElem for Or<T, U> {
    fn elem_symbol(_t: PhantomData<Self>) -> EnumSet<ElemSymbol> {
        let t_set = <T as AnElem>::elem_symbol(PhantomData);
        let u_set = <U as AnElem>::elem_symbol(PhantomData);
        t_set.union(u_set)
    }

    fn to_elem(self) -> Elem {
        match self {
            Self::Left(x) => x.to_elem(),
            Self::Right(x) => x.to_elem(),
        }
    }

    fn from_elem(_t: PhantomData<Self>, x: Elem) -> Result<Self, StackError> {
        AnElem::from_elem(PhantomData::<T>, x.clone())
            .map(|y| Or::Left(y))
            .or_else(|e_hd| {
               Ok(Or::Right(AnElem::from_elem(PhantomData::<U>, x)?))
                   .map_err(|e_tl| {
                       StackError::PopOr {
                           e_hd: Box::new(e_hd),
                           e_tl: Box::new(e_tl),
                       }})
            })
    }
}




impl<T: AnElem, U: Elems> Elems for Or<T, U> {
    type Hd = T;
    type Tl = U;

    fn is_left(&self) -> bool {
        match self {
            Self::Left(_) => true,
            Self::Right(_) => false,
        }
    }

    fn left(_s: PhantomData<Self>, x: Self::Hd) -> Self {
        Self::Left(x)
    }

    fn right(_s: PhantomData<Self>, x: Self::Tl) -> Self {
        Self::Right(x)
    }

    fn or<V, F: Fn(Self::Hd) -> V, G: Fn(Self::Tl) -> V>(&self, f: F, g: G) -> V {
        match self {
            Self::Left(x) => f(x.clone()),
            Self::Right(x) => g(x.clone()),
        }
    }
}








// fn cons<U, V: AnElem + Trait<U, V>>(self, u: PhantomData<U>, x: V) -> ConsT<U, V, Self> where Self: Sized;
#[derive(Clone, PartialEq, Eq)]
pub struct Cons<T: AnElem, N: ArrayLength<T>, U: IList> {
    hd: GenericArray<T, N>,
    tl: U,
}

pub struct IterCons<T: AnElem, N: ArrayLength<T>, U: IList> {
    hd: GenericArrayIter<T, N>,
    cons: <U as IntoIterator>::IntoIter,
}

impl<T: AnElem, N: ArrayLength<T>, U: IList> IntoIterator for Cons<T, N, U> {
    type Item = Elem;
    type IntoIter = IterCons<T, N, U>;

    fn into_iter(self) -> Self::IntoIter {
        IterCons {
            hd: self.hd.into_iter(),
            cons: self.tl.into_iter(),
        }
    }
}

impl<T: AnElem, N: ArrayLength<T>, U: IList> Iterator for IterCons<T, N, U> {
    type Item = Elem;

    fn next(&mut self) -> Option<Self::Item> {
        self.hd.next()
            .map(|x| x.to_elem())
            .or_else(|| self.cons.next())
    }
}




pub trait IList: Clone + IntoIterator<Item = Elem> {
    type Hd: AnElem;
    type N: ArrayLength<Self::Hd>;
    type Tl: IList;

    fn is_empty(&self) -> bool;
    fn hd(&self) -> GenericArray<Self::Hd, Self::N>;
    fn tl(&self) -> Self::Tl;
    fn cons<T: AnElem, M: ArrayLength<T>>(self, x: GenericArray<T, M>) -> Cons<T, M, Self> where Self: Sized;
    fn pop(x: PhantomData<Self>, stack: &mut Stack) -> Result<Self, StackError>;
}

impl IList for Nil {
    type Hd = ();
    type N = U0;
    type Tl = Nil;

    fn is_empty(&self) -> bool {
        true
    }

    fn hd(&self) -> GenericArray<Self::Hd, Self::N> {
        GenericArray::generate(|_| ())
    }

    fn tl(&self) -> Self::Tl {
        Self {}
    }

    fn cons<T: AnElem, M: ArrayLength<T>>(self, x: GenericArray<T, M>) -> Cons<T, M, Self>
    where
        Self: Sized,
    {
        Cons {
            hd: x,
            tl: self,
        }
    }

    fn pop(_x: PhantomData<Self>, _stack: &mut Stack) -> Result<Self, StackError> {
        Ok(Nil {})
    }
}

impl<T: AnElem, N: ArrayLength<T>, U: IList> IList for Cons<T, N, U> {
    type Hd = T;
    type N = N;
    type Tl = U;

    fn is_empty(&self) -> bool {
        false
    }

    fn hd(&self) -> GenericArray<Self::Hd, Self::N> {
        self.hd.clone()
    }

    fn tl(&self) -> Self::Tl {
        self.tl.clone()
    }

    fn cons<V: AnElem, M: ArrayLength<V>>(self, x: GenericArray<V, M>) -> Cons<V, M, Self>
    where
        Self: Sized,
    {
        Cons {
            hd: x,
            tl: self,
        }
    }

    // TODO: add better errors
    fn pop(_x: PhantomData<Self>, stack: &mut Stack) -> Result<Self, StackError> {
        let hd_arr = stack.pop_generic_array(PhantomData, PhantomData)?;
        Ok(Cons {
            hd: hd_arr,
            tl: Self::Tl::pop(PhantomData, stack)?,
        })
    }
}

impl Stack {
    // TODO: reversed?
    pub fn pop_generic_array<T: AnElem, N: ArrayLength<T>>(&mut self,
                                                           _t: PhantomData<T>,
                                                           _n: PhantomData<N>) -> Result<GenericArray<T, N>, StackError> {
        let mut xs = vec![];
        for _current_index in 1..<N as Unsigned>::USIZE {
            let hd_elem = self.pop()?;
            xs.push(AnElem::from_elem(PhantomData::<T>, hd_elem)?)
        }
        GenericArray::from_exact_iter(xs).ok_or_else(|| StackError::TODO)
    }
}


pub trait IOList: IList {
    type Return: AnElem;
}

#[derive(Clone, PartialEq, Eq)]
pub struct ConsOut<T: AnElem, N: ArrayLength<T>, U: IList> {
    cons: Cons<T, N, U>,
}

impl<T: AnElem, N: ArrayLength<T>, U: IList> IntoIterator for ConsOut<T, N, U> {
    type Item = Elem;
    type IntoIter = IterCons<T, N, U>;

    fn into_iter(self) -> Self::IntoIter {
      self.cons.into_iter()
    }
}

impl<T: AnElem, N: ArrayLength<T>, U: IList> IList for ConsOut<T, N, U> {
    type Hd = T;
    type N = N;
    type Tl = U;

    fn is_empty(&self) -> bool {
        self.cons.is_empty()
    }

    fn hd(&self) -> GenericArray<Self::Hd, Self::N> {
        self.cons.hd()
    }

    fn tl(&self) -> Self::Tl {
        self.cons.tl()
    }

    fn cons<V: AnElem, M: ArrayLength<V>>(self, x: GenericArray<V, M>) -> Cons<V, M, Self>
    where
        Self: Sized,
    {
        Cons {
            hd: x,
            tl: self,
        }
    }

    fn pop(_x: PhantomData<Self>, stack: &mut Stack) -> Result<Self, StackError> {
      Ok(ConsOut {
        cons: IList::pop(PhantomData, stack)?,
      })
    }
}

impl<T: AnElem, N: ArrayLength<T>, U: IList> IOList for ConsOut<T, N, U> {
    type Return = T;
}

impl<T: AnElem, N: ArrayLength<T>, U: IOList> IOList for Cons<T, N, U> {
    type Return = <U as IOList>::Return;
}

pub trait IsInstructionT: std::fmt::Debug {
    type In: IOList;
    type Error: AnError;

    fn run(&self, x: Self::In) -> Result<<Self::In as IOList>::Return, Self::Error>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Concat<T: AnElem> {
    t: PhantomData<T>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ConcatError {}
impl AnError for ConcatError {}

impl<T: AnElem + IntoIterator + FromIterator<<T as IntoIterator>::Item>> IsInstructionT for Concat<T> {
    type In = ConsOut<T, U2, Nil>;
    type Error = ConcatError;

    fn run(&self, x: Self::In) -> Result<<Self::In as IOList>::Return, Self::Error> {
        let lhs = x.hd()[0].clone();
        let rhs = x.hd()[1].clone();
        Ok(lhs.into_iter().chain(rhs.into_iter()).collect())
    }
}








