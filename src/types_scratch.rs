use crate::elem::{Elem, AnElem, AnElemError, ElemSymbol};
use crate::stack::{Stack, StackError};
use crate::types::{AnError, Nil};

use std::iter::{FromIterator};
use std::marker::PhantomData;
use std::fmt::Debug;
use std::sync::Arc;

use enumset::EnumSet;
use generic_array::typenum::{U0, U2};
use generic_array::sequence::GenericSequence;
use generic_array::{arr, GenericArray, GenericArrayIter, ArrayLength};
// use typenum::marker_traits::Unsigned;
use serde_json::{Map, Value};

// NEXT:
// - Acheive parity between ElemList -> Elems/IList/IOList

// pub trait IList: Clone + IntoIterator<Item = Elem> {
//     type Hd: AnElem;
//     type N: ArrayLength<Self::Hd>;
//     type Tl: IList;

//     fn is_empty(&self) -> bool;
//     fn hd(&self) -> GenericArray<Self::Hd, Self::N>;
//     fn tl(&self) -> Self::Tl;
//     fn cons<T: AnElem, M: ArrayLength<T>>(self, x: GenericArray<T, M>) -> Cons<T, M, Self> where Self: Sized;
//     fn pop(x: PhantomData<Self>, stack: &mut Stack) -> Result<Self, StackError>;


// pub trait AnElem: Clone + std::fmt::Debug {
//     fn elem_symbol(t: PhantomData<Self>) -> ElemSymbol;
//     fn elem_symbol(t: PhantomData<Self>) -> EnumSet<ElemSymbol>;

//     fn to_elem(self) -> Elem;

//     fn from_elem(t: PhantomData<Self>, x: Elem) -> Result<Self, AnElemError>;
// }


pub trait Elems: Clone + Debug {
    type Hd: AnElem;
    type N: ArrayLength<Self::Hd>;
    type Tl: Elems<N = Self::N>;

    fn is_left(&self) -> bool;
    fn left(s: PhantomData<Self>, x: GenericArray<Self::Hd, Self::N>) -> Self;
    fn right(s: PhantomData<Self>, x: Self::Tl) -> Self;
    fn or<T, F: Fn(GenericArray<Self::Hd, Self::N>) -> T, G: Fn(Self::Tl) -> T>(&self, f: F, g: G) -> T;

    // fn elem_symbols(t: PhantomData<Self>) -> EnumSet<ElemSymbol>;
    // fn to_elems(self) -> Elem;
    // fn from_elems(t: PhantomData<Self>, x: &mut Stack) -> Result<Self, ElemsError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Singleton<T, N>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
{
    t: GenericArray<T, N>,
}

// impl<T: AnElem> AnElem for Singleton<T> {
//     fn elem_symbol(_t: PhantomData<Self>) -> EnumSet<ElemSymbol> {
//         <T as AnElem>::elem_symbol(PhantomData)
//     }

//     fn to_elem(self) -> Elem {
//         self.t.to_elem()
//     }

//     fn from_elem(_t: PhantomData<Self>, x: Elem) -> Result<Self, AnElemError> {
//         <T as AnElem>::from_elem(PhantomData, x).map(|y| {
//             Singleton {
//                 t: y,
//             }
//         })
//     }
// }

impl<T, N> Elems for Singleton<T, N>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
{
    type Hd = T;
    type N = N;
    type Tl = Singleton<T, N>;

    fn is_left(&self) -> bool { true }

    fn left(_s: PhantomData<Self>, x: GenericArray<Self::Hd, Self::N>) -> Self {
        Singleton {
            t: x,
        }
    }

    fn right(_s: PhantomData<Self>, x: Self::Tl) -> Self {
        x
    }

    fn or<U, F: Fn(GenericArray<Self::Hd, Self::N>) -> U, G: Fn(Self::Tl) -> U>(&self, f: F, _g: G) -> U {
        f(self.t.clone())
    }
}


#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Or<T, N, U>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
    U: Elems<N = N>,
{
    Left(GenericArray<T, N>),
    Right(U),
}

// impl<T: AnElem, U: Elems> AnElem for Or<T, U> {
//     fn elem_symbol(_t: PhantomData<Self>) -> EnumSet<ElemSymbol> {
//         let t_set = <T as AnElem>::elem_symbol(PhantomData);
//         let u_set = <U as AnElem>::elem_symbol(PhantomData);
//         t_set.union(u_set)
//     }

//     fn to_elem(self) -> Elem {
//         match self {
//             Self::Left(x) => x.to_elem(),
//             Self::Right(x) => x.to_elem(),
//         }
//     }

//     fn from_elem(_t: PhantomData<Self>, x: Elem) -> Result<Self, AnElemError> {
//         AnElem::from_elem(PhantomData::<T>, x.clone())
//             .map(|y| Or::Left(y))
//             .or_else(|e_hd| {
//                Ok(Or::Right(AnElem::from_elem(PhantomData::<U>, x)?))
//                    .map_err(|e_tl| {
//                        AnElemError::PopOr {
//                            e_hd: Box::new(e_hd),
//                            e_tl: Box::new(e_tl),
//                        }})
//             })
//     }
// }

impl<T, N, U> Elems for Or<T, N, U>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
    U: Elems<N = N>,
{
    type Hd = T;
    type N = N;
    type Tl = U;

    fn is_left(&self) -> bool {
        match self {
            Self::Left(_) => true,
            Self::Right(_) => false,
        }
    }

    fn left(_s: PhantomData<Self>, x: GenericArray<Self::Hd, Self::N>) -> Self {
        Self::Left(x)
    }

    fn right(_s: PhantomData<Self>, x: Self::Tl) -> Self {
        Self::Right(x)
    }

    fn or<V, F: Fn(GenericArray<Self::Hd, Self::N>) -> V, G: Fn(Self::Tl) -> V>(&self, f: F, g: G) -> V {
        match self {
            Self::Left(x) => f(x.clone()),
            Self::Right(x) => g(x.clone()),
        }
    }
}






// + IntoIterator<Item = Elem>
pub trait IsList: Clone {
    type Hd: Elems;
    type Tl: IsList;

    fn is_empty(&self) -> bool;
    fn hd(self) -> Self::Hd;
    fn tl(self) -> Self::Tl;
    fn cons<T: Elems>(self, x: T) -> Cons<T, Self> {
        Cons {
            hd: x,
            tl: self,
        }
    }

    // fn pop(x: PhantomData<Self>, stack: &mut Stack) -> Result<Self, StackError>;
}

impl IsList for Nil {
    type Hd = Singleton<(), U0>;
    type Tl = Nil;

    fn is_empty(&self) -> bool {
        true
    }

    fn hd(self) -> Self::Hd {
        Singleton {
            t: GenericArray::generate(|_| ()),
        }
    }

    fn tl(self) -> Self::Tl {
        Self {}
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Cons<T: Elems, U: IsList> {
    hd: T,
    tl: U,
}

impl<T: Elems, U: IsList> IsList for Cons<T, U> {
    type Hd = T;
    type Tl = U;

    fn is_empty(&self) -> bool {
        false
    }

    fn hd(self) -> Self::Hd {
        self.hd
    }

    fn tl(self) -> Self::Tl {
        self.tl
    }

    // // TODO: add better errors
    // fn pop(_x: PhantomData<Self>, stack: &mut Stack) -> Result<Self, StackError> {
    //     let hd_arr = stack.pop_generic_array(PhantomData, PhantomData)?;
    //     Ok(Cons {
    //         hd: hd_arr,
    //         tl: Self::Tl::pop(PhantomData, stack)?,
    //     })
    // }
}






pub trait IList: IsList {
}

impl IList for Nil {
}

impl<T: Elems, U: IList> IList for Cons<T, U> {
}



/// return_value is private, but get_return_value can be used to extract it
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Return<T, U: AnElem> {
    for_instruction: PhantomData<T>,
    return_value: U,
}

impl<T, U: AnElem> Return<T, U> {
    pub fn get_return_value(self) -> U {
        self.return_value
    }
}


pub trait IOList: IsList {
    type Return: AnElem;

    fn returning(self) -> Option<Arc<dyn FnOnce(Self::Return) -> Return<(), Self::Return>>>;
}

#[derive(Clone)]
pub enum ConsOut<T, N, U, V>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
    U: Elems<N = N>,
    V: IList,
{
    Left {
        return_fn: Arc<dyn FnOnce(T) -> Return<(), T>>,
        hd: Singleton<T, N>,
        tl: V,
    },
    Right {
        hd: U,
        tl: V,
    }
}

impl<T, N, U, V> IsList for ConsOut<T, N, U, V>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
    U: Elems<N = N>,
    V: IList,
{
    type Hd = Or<T, N, U>;
    type Tl = V;

    fn is_empty(&self) -> bool {
        false
    }

    fn hd(self) -> Self::Hd {
        match self {
            Self::Left { hd, .. } => Or::Left(hd.t),
            Self::Right { hd, .. } => Or::Right(hd),
        }
    }

    fn tl(self) -> Self::Tl {
        match self {
            Self::Left { tl, .. } => tl,
            Self::Right { tl, .. } => tl,
        }
    }
}

impl<T, N, U, V> IOList for ConsOut<T, N, U, V>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
    U: Elems<N = N>,
    V: IList,
{
    type Return = T;

    fn returning(self) -> Option<Arc<dyn FnOnce(T) -> Return<(), T>>> {
        match self {
            Self::Left { return_fn, .. } => Some(return_fn),
            Self::Right { .. } => None,
        }
    }
}

impl<T, U> IOList for Cons<T, U>
where
    T: Elems,
    U: IOList,
{
    type Return = U::Return;

    fn returning(self) -> Option<Arc<dyn FnOnce(U::Return) -> Return<(), U::Return>>> {
        self.tl.returning()
    }
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


//// bytes, string, array, object
// impl IsInstructionT for Concat<()> {
//     type In = ConsOut<Or<Vec<Value>, Singleton<Map<String, Value>>>, U2, Nil>;
//     type Error = ConcatError;

//     fn run(&self, x: Self::In) -> Result<<Self::In as IOList>::Return, Self::Error> {
//         let lhs: Or<Vec<Value>, Singleton<Map<String, Value>>> = x.hd()[0].clone();
//         let rhs: Or<Vec<Value>, Singleton<Map<String, Value>>> = x.hd()[1].clone();
//         Ok(lhs.into_iter().chain(rhs.into_iter()).collect())
//     }
// }


// impl<T: AnElem + IntoIterator + FromIterator<<T as IntoIterator>::Item>> IsInstructionT for Concat<T> {
//     type In = ConsOut<T, U2, Nil>;
//     type Error = ConcatError;

//     fn run(&self, x: Self::In) -> Result<<Self::In as IOList>::Return, Self::Error> {
//         let lhs = x.hd()[0].clone();
//         let rhs = x.hd()[1].clone();
//         Ok(lhs.into_iter().chain(rhs.into_iter()).collect())
//     }
// }

