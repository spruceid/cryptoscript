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
pub trait IList: Clone {
    type Hd: Elems;
    // type N: ArrayLength<Self::Hd>;
    type Tl: IList;

    fn is_empty(&self) -> bool;
    // fn hd(&self) -> GenericArray<Self::Hd, Self::N>;
    fn hd(self) -> Self::Hd;
    fn tl(self) -> Self::Tl;
    // fn cons<T: AnElem, M: ArrayLength<T>>(self, x: GenericArray<T, M>) -> Cons<T, M, Self> where Self: Sized;
    fn cons<T: Elems>(self, x: T) -> Cons<T, Self> {
        Cons {
            hd: x,
            tl: self,
        }
    }
    // where Self: Sized;

    // fn pop(x: PhantomData<Self>, stack: &mut Stack) -> Result<Self, StackError>;
}

// fn cons<U, V: AnElem + Trait<U, V>>(self, u: PhantomData<U>, x: V) -> ConsT<U, V, Self> where Self: Sized;
#[derive(Clone, PartialEq, Eq)]
// pub struct Cons<T: AnElem, N: ArrayLength<T>, U: IList> {
//     hd: GenericArray<T, N>,
//     tl: U,
// }
pub struct Cons<T: Elems, U: IList> {
    hd: T,
    tl: U,
}

// pub struct IterCons<T: AnElem, N: ArrayLength<T>, U: IList> {
//     hd: GenericArrayIter<T, N>,
//     cons: <U as IntoIterator>::IntoIter,
// }

// impl<T: AnElem, N: ArrayLength<T>, U: IList> IntoIterator for Cons<T, N, U> {
//     type Item = Elem;
//     type IntoIter = IterCons<T, N, U>;

//     fn into_iter(self) -> Self::IntoIter {
//         IterCons {
//             hd: self.hd.into_iter(),
//             cons: self.tl.into_iter(),
//         }
//     }
// }

// impl<T: AnElem, N: ArrayLength<T>, U: IList> Iterator for IterCons<T, N, U> {
//     type Item = Elem;

//     fn next(&mut self) -> Option<Self::Item> {
//         self.hd.next()
//             .map(|x| x.to_elem())
//             .or_else(|| self.cons.next())
//     }
// }

impl IList for Nil {
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

    // fn pop(_x: PhantomData<Self>, _stack: &mut Stack) -> Result<Self, StackError> {
    //     Ok(Nil {})
    // }
}

impl<T: Elems, U: IList> IList for Cons<T, U> {
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














/// return_value is private, but get_return_value can be used to extract it
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Return<T: AnElem> {
    return_value: T,
}

impl<T: AnElem> Return<T> {
    pub fn get_return_value(self) -> T {
        self.return_value
    }
}

pub trait IOList: IList {
    type Return: AnElem;
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
        return_fn: Arc<dyn Fn(T) -> Return<T>>,
        hd: Singleton<T, N>,
        tl: V,
    },
    Right {
        hd: U,
        tl: V,
    }
}

impl<T, N, U, V> IList for ConsOut<T, N, U, V>
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

    fn returning(&self) -> Arc<dyn Fn(T) -> Return<T>>;
}

impl<T, U> IOList for Cons<T, U>
where
    T: Elems,
    U: IOList,
{
    type Return = U::Return;
}

// impl<T: AnElem, N: ArrayLength<T>, U: IList> IntoIterator for ConsOut<T, N, U> {
//     type Item = Elem;
//     type IntoIter = IterCons<T, N, U>;

//     fn into_iter(self) -> Self::IntoIter {
//       self.cons.into_iter()
//     }
// }

// impl<T: AnElem, N: ArrayLength<T>, U: IList> IList for ConsOut<T, N, U> {
//     type Hd = T;
//     type N = N;
//     type Tl = U;

//     fn is_empty(&self) -> bool {
//         self.cons.is_empty()
//     }

//     fn hd(&self) -> GenericArray<Self::Hd, Self::N> {
//         self.cons.hd()
//     }

//     fn tl(&self) -> Self::Tl {
//         self.cons.tl()
//     }

//     fn cons<V: AnElem, M: ArrayLength<V>>(self, x: GenericArray<V, M>) -> Cons<V, M, Self>
//     where
//         Self: Sized,
//     {
//         Cons {
//             hd: x,
//             tl: self,
//         }
//     }

//     fn pop(_x: PhantomData<Self>, stack: &mut Stack) -> Result<Self, StackError> {
//       Ok(ConsOut {
//         cons: IList::pop(PhantomData, stack)?,
//       })
//     }
// }

// impl<T: AnElem, N: ArrayLength<T>, U: IList> IOList for ConsOut<T, N, U> {
//     type Return = T;
// }

// impl<T: AnElem, N: ArrayLength<T>, U: IOList> IOList for Cons<T, N, U> {
//     type Return = <U as IOList>::Return;
// }

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

// pub trait FromIntoIterator: IntoIterator + FromIterator<<Self as IntoIterator>::Item> {}

// impl<T> IntoIterator for Singleton<T>
// where
//     T: AnElem + IntoIterator,
// {
//     type Item = <T as IntoIterator>::Item;
//     type IntoIter = <T as IntoIterator>::IntoIter;

//     fn into_iter(self) -> Self::IntoIter {
//         self.t.into_iter()
//     }
// }

// impl<T> FromIterator<<T as IntoIterator>::Item> for Singleton<T>
// where
//     T: AnElem + IntoIterator + FromIterator<<T as IntoIterator>::Item>,
// {
//     fn from_iter<U>(iter: U) -> Self
//     where
//         U: IntoIterator<Item = <T as IntoIterator>::Item>,
//     {
//         Singleton {
//             t: <T as FromIterator<<T as IntoIterator>::Item>>::from_iter(iter),
//         }
//     }
// }

// pub enum OrIter<T, U>
// where
//     T: AnElem + IntoIterator,
//     U: Elems + IntoIterator,
// {
//     Left(<T as IntoIterator>::IntoIter),
//     Right(<U as IntoIterator>::IntoIter),
// }

// pub enum OrIterItem<T, U>
// where
//     T: AnElem + IntoIterator,
//     U: Elems + IntoIterator,
// {
//     Left(<T as IntoIterator>::Item),
//     Right(<U as IntoIterator>::Item),
// }

// impl<T, U> Iterator for OrIter<T, U>
// where
//     T: AnElem + IntoIterator,
//     U: Elems + IntoIterator,
// {
//     type Item = OrIterItem<T, U>;

//     fn next(&mut self) -> Option<Self::Item> {
//         match self {
//             Self::Left(x) => x.next().map(|y| OrIterItem::Left(y)),
//             Self::Right(x) => x.next().map(|y| OrIterItem::Right(y)),
//         }
//     }
// }

// impl<T, U> IntoIterator for Or<T, U>
// where
//     T: AnElem + IntoIterator,
//     U: Elems + IntoIterator,
// {
//     type Item = OrIterItem<T, U>;
//     type IntoIter = OrIter<T, U>;

//     fn into_iter(self) -> Self::IntoIter {
//         match self {
//             Self::Left(x) => OrIter::Left(x.into_iter()),
//             Self::Right(x) => OrIter::Right(x.into_iter()),
//         }
//     }
// }


// pub struct ResultOrIterError<T> {
//     result: Result<T, OrIterError>,
// }

// pub enum OrIterError {
//     FromEmptyIter,

//     AnElemError(AnElemError),

//     MoreThanSingleton {
//         // hd_elem: String,
//         // other_elems: Vec<Elem>,
//     },
// }

// impl<T> IntoIterator for ResultOrIterError<T>
// where
//     T: IntoIterator,
// {
//     type Item = <Option<T> as IntoIterator>::Item;
//     type IntoIter = <Option<T> as IntoIterator>::IntoIter;

//     fn into_iter(self) -> Self::IntoIter {
//         match self.result {
//             Ok(x) => Some(x).into_iter(),
//             Err(_) => None.into_iter(),
//         }
//     }
// }

// // impl<t, u> fromiterator<oriteritem<t, u>> for resultoritererror<or<t, u>>
// // where
// //     T: AnElem + IntoIterator + FromIterator<<T as IntoIterator>::Item>,
// //     U: Elems + IntoIterator + FromIterator<<U as IntoIterator>::Item>,
// // {
// //     fn from_iter<V>(iter: V) -> Self
// //     where
// //         V: IntoIterator<Item = OrIterItem<T, U>>,
// //     {
// //         // let mut iter_mut = iter.into_iter();
// //         // let opt_hd_elem = iter_mut.next();
// //         // let rest = iter_mut.next();
// //         // ResultOrIterError {
// //         //     result: match (opt_hd_elem, rest) {
// //         //         (None, _) => Err(OrIterError::FromEmptyIter),
// //         //         (Some(hd_elem), Some(other_elems)) => Err(OrIterError::MoreThanSingleton {
// //         //             // TODO: debug info
// //         //             // hd_elem: hd_elem,
// //         //             // other_elems: other_elems,
// //         //         }),
// //         //         (Some(OrIterItem::Left(hd_elem)), None) =>
// //         //             match <Or<T, U> as AnElem>::from_elem(PhantomData, hd_elem) {
// //         //                 Ok(x) => Ok(x),
// //         //                 Err(e) => Err(OrIterError::AnElemError(e)),
// //         //             },
// //         //         (Some(OrIterItem::Right(hd_elem)), None) =>
// //         //             match <Or<T, U> as AnElem>::from_elem(PhantomData, hd_elem) {
// //         //                 Ok(x) => Ok(x),
// //         //                 Err(e) => Err(OrIterError::AnElemError(e)),
// //         //             },
// //         //     },
// //         // }
// //     }
// // }

// // impl<T, U> FromIterator<OrIterItem<T, U>> for OrIterFrom<T, U>
// // where
// //     T: AnElem + IntoIterator + FromIterator<<T as IntoIterator>::Item>,
// //     U: Elems + IntoIterator + FromIterator<<U as IntoIterator>::Item>,
// // {
// //     fn from_iter<V>(iter: V) -> Self
// //     where
// //         V: IntoIterator<Item = OrIterItem<T, U>>,
// //     {
// //         let mut iter_mut = iter.into_iter();
// //         let opt_hd_elem = iter_mut.next();
// //         let rest = iter_mut.next();
// //         match (opt_hd_elem, rest) {
// //             (None, _) => OrIterFrom::Err(OrIterError::FromEmptyIter),
// //             (Some(hd_elem), Some(other_elems)) => OrIterFrom::Err(OrIterError::MoreThanSingleton {
// //                 // TODO: debug info
// //                 // hd_elem: hd_elem,
// //                 // other_elems: other_elems,
// //             }),
// //             (Some(OrIterItem::Left(hd_elem)), None) =>
// //                 match <Or<T, U> as AnElem>::from_elem(PhantomData, hd_elem) {
// //                     Ok(x) => OrIterFrom::Or(x),
// //                     Err(e) => OrIterFrom::Err(OrIterError::AnElemError(e)),
// //                 },
// //             (Some(OrIterItem::Right(hd_elem)), None) =>
// //                 match <Or<T, U> as AnElem>::from_elem(PhantomData, hd_elem) {
// //                     Ok(x) => OrIterFrom::Or(x),
// //                     Err(e) => OrIterFrom::Err(OrIterError::AnElemError(e)),
// //                 },
// //         }
// //     }
// // }



// // impl<T, U> FromIterator<<T as IntoIterator>::Item> for Or<T, U>
// // where
// //     T: AnElem + IntoIterator + FromIterator<<T as IntoIterator>::Item>,
// //     U: Elems + IntoIterator<Item = <T as IntoIterator>::Item> + FromIterator<<T as IntoIterator>::Item>,
// // {
// //     fn from_iter<V>(iter: V) -> Self
// //     where
// //         V: IntoIterator<Item = <T as IntoIterator>::Item>,
// //     {
// //         _
// //     }
// // }








