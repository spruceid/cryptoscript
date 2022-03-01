use crate::elem::{Elem};
use crate::stack::{Stack, StackError};
use crate::types::{Empty, AnElem, AnError, Nil, Teq, TEq, TypeName};

use std::iter::{FromIterator};

// use std::fmt;
// use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
// use std::sync::Arc;

use generic_array::{GenericArray, ArrayLength};


// fn cons<U, V: AnElem + Trait<U, V>>(self, u: PhantomData<U>, x: V) -> ConsT<U, V, Self> where Self: Sized;
#[derive(Clone, PartialEq, Eq)]
pub struct Cons<T: AnElem, U: IList> {
    hd: T,
    tl: U,
}

#[derive(Clone, PartialEq, Eq)]
pub struct IterCons<T: AnElem, U: IList> {
    cons: Cons<T, U>,
    at_head: bool,
}

impl<T: AnElem, U: IList> IntoIterator for Cons<T, U> {
    type Item = Elem;
    type IntoIter = IterCons<T, U>;

    fn into_iter(self) -> Self::IntoIter {
        IterCons {
            cons: self,
            at_head: true,
        }
    }
}

impl<T: AnElem, U: IList> Iterator for IterCons<T, U> {
    type Item = Elem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.at_head {
            Some(AnElem::is_elem(PhantomData::<T>).to_elem(self.cons.hd.clone()))
        } else {
            let self_cons = self.cons.clone();
            *self = self_cons.into_iter();
            self.next()
        }
    }
}




pub trait IList: Clone + IntoIterator<Item = Elem> {
    type Hd: AnElem;
    // type N: ArrayLength<Self::Hd>;
    type Tl: IList;

    fn is_empty(&self) -> bool;
    fn hd(&self) -> Self::Hd;
    fn tl(&self) -> Self::Tl;
    fn cons<T: AnElem>(self, x: T) -> Cons<T, Self> where Self: Sized;
    fn pop(x: PhantomData<Self>, stack: &mut Stack) -> Result<Self, StackError>;
}

impl IList for Nil {
    type Hd = ();
    type Tl = Nil;

    fn is_empty(&self) -> bool {
        true
    }

    fn hd(&self) -> Self::Hd {
        ()
    }

    fn tl(&self) -> Self::Tl {
        Self {}
    }

    fn cons<T: AnElem>(self, x: T) -> Cons<T, Self>
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

impl<T: AnElem, U: IList> IList for Cons<T, U> {
    type Hd = T;
    type Tl = U;

    fn is_empty(&self) -> bool {
        false
    }

    fn hd(&self) -> Self::Hd {
        self.hd.clone()
    }

    fn tl(&self) -> Self::Tl {
        self.tl.clone()
    }

    fn cons<V: AnElem>(self, x: V) -> Cons<V, Self>
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
        let hd_elem = stack.pop()?;
        Ok(Cons {
            hd: AnElem::is_elem(PhantomData::<Self::Hd>).from_elem(hd_elem.clone()).ok_or_else(|| StackError::UnexpectedElemType {
                expected: AnElem::is_elem(PhantomData::<Self::Hd>).elem_symbol(),
                found: hd_elem.clone(),
                stack: stack.clone(),
            })?,
            tl: Self::Tl::pop(PhantomData, stack)?,
        })
    }

}

pub trait IOList: IList {
    type Return: AnElem;
}

#[derive(Clone, PartialEq, Eq)]
pub struct ConsOut<T: AnElem, U: IList> {
    cons: Cons<T, U>,
}

impl<T: AnElem, U: IList> IntoIterator for ConsOut<T, U> {
    type Item = Elem;
    type IntoIter = IterCons<T, U>;

    fn into_iter(self) -> Self::IntoIter {
      self.cons.into_iter()
    }
}

impl<T: AnElem, U: IList> IList for ConsOut<T, U> {
    type Hd = T;
    type Tl = U;

    fn is_empty(&self) -> bool {
        self.cons.is_empty()
    }

    fn hd(&self) -> Self::Hd {
        self.cons.hd()
    }

    fn tl(&self) -> Self::Tl {
        self.cons.tl()
    }

    fn cons<V: AnElem>(self, x: V) -> Cons<V, Self>
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

impl<T: AnElem, U: IList> IOList for ConsOut<T, U> {
    type Return = T;
}

impl<T: AnElem, U: IOList> IOList for Cons<T, U> {
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
    type In = ConsOut<T, Nil>;
    // type In = (T, T);
    // type Out = T;
    // type Error = Empty;
    type Error = ConcatError;

    fn run(&self, x: Self::In) -> Result<<Self::In as IOList>::Return, Self::Error> {
        // let (lhs, rhs) = x;
        // Ok(lhs.into_iter().chain(rhs.into_iter()).collect())
        Err(ConcatError {})
    }
}








