use crate::elem::{Elem, AnElem, AnElemError, ElemSymbol};
use crate::stack::{Stack, StackError};
use crate::types::{AnError, Nil};

use std::iter::{FromIterator};
use std::marker::PhantomData;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use enumset::EnumSet;
use generic_array::typenum::{U0, U2};
use generic_array::sequence::GenericSequence;
use generic_array::{arr, GenericArray, GenericArrayIter, ArrayLength};
use serde_json::{Map, Value};

// NEXT:
// - Acheive parity between ElemList -> Elems/IList/IOList

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Singleton<T, N>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
{
    array: GenericArray<T, N>,
}

// impl<T: AnElem> AnElem for Singleton<T> {
//     fn elem_symbol(_t: PhantomData<Self>) -> EnumSet<ElemSymbol> { <T as AnElem>::elem_symbol(PhantomData) }
//     fn to_elem(self) -> Elem { self.t.to_elem() }
//     fn from_elem(_t: PhantomData<Self>, x: Elem) -> Result<Self, AnElemError> { <T as AnElem>::from_elem(PhantomData, x).map(|y| { Singleton { t: y, } }) }
// }

pub trait Elems: Clone + Debug {
    type Hd: AnElem;
    type N: ArrayLength<Self::Hd>;
    type Tl: Elems<N = Self::N>;

    // fn left(s: PhantomData<Self>, x: GenericArray<Self::Hd, Self::N>) -> Self;
    // fn right(s: PhantomData<Self>, x: Self::Tl) -> Self;
    fn or<T, F: Fn(GenericArray<Self::Hd, Self::N>) -> T, G: Fn(Self::Tl) -> T>(&self, f: F, g: G) -> T;

    // fn elem_symbols(t: PhantomData<Self>) -> EnumSet<ElemSymbol>;
    // fn to_elems(self) -> Elem;
    // fn from_elems(t: PhantomData<Self>, x: &mut Stack) -> Result<Self, ElemsError>;
}

pub trait IElems: Elems {}

pub trait IOElems<'a>: Elems {
    fn or_return<T, F, G>(&'a self, f: &mut F, g: &mut G) -> T
        where
            F: Fn(GenericArray<Self::Hd, Self::N>, &'a mut Option<Self::Hd>) -> T,
            G: Fn(Self::Tl) -> T;
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
    fn or<U, F: Fn(GenericArray<Self::Hd, Self::N>) -> U, G: Fn(Self::Tl) -> U>(&self, f: F, _g: G) -> U {
        f(self.array.clone())
    }
}

impl<T, N> IElems for Singleton<T, N>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
{}



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

impl<T, N, U> Elems for Or<T, N, U>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
    U: Elems<N = N>,
{
    type Hd = T;
    type N = N;
    type Tl = U;

    // fn left(_s: PhantomData<Self>, x: GenericArray<Self::Hd, Self::N>) -> Self { Self::Left(x) }
    // fn right(_s: PhantomData<Self>, x: Self::Tl) -> Self { Self::Right(x) }
    fn or<V, F: Fn(GenericArray<Self::Hd, Self::N>) -> V, G: Fn(Self::Tl) -> V>(&self, f: F, g: G) -> V {
        match self {
            Self::Left(x) => f(x.clone()),
            Self::Right(x) => g(x.clone()),
        }
    }
}

impl<T, N, U> IElems for Or<T, N, U>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
    U: IElems<N = N>,
{}



#[derive(Clone, Debug)]
pub struct ReturnSingleton<'a, T, N>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
{
    singleton: Singleton<T, N>,
    returning: Arc<Mutex<&'a mut Option<T>>>,
}

impl<'a, T, N> Elems for ReturnSingleton<'a, T, N>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
{
    type Hd = T;
    type N = N;
    type Tl = Singleton<T, N>;

    // fn left(_s: PhantomData<Self>, x: GenericArray<Self::Hd, Self::N>) -> Self { Elems::left(PhantomData::<Singleton<T, N>>, x) }
    // fn right(_s: PhantomData<Self>, x: Self::Tl) -> Self { Elems::left(PhantomData::<Singleton<T, N>>, x) }
    fn or<U, F: Fn(GenericArray<Self::Hd, Self::N>) -> U, G: Fn(Self::Tl) -> U>(&self, f: F, g: G) -> U {
        self.singleton.or(f, g)
    }
}

impl<'a, T, N> IOElems<'a> for ReturnSingleton<'a, T, N>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
{
    fn or_return<U, F, G>(&'a self, f: &mut F, _g: &mut G) -> U
    where
        F: Fn(GenericArray<Self::Hd, Self::N>, &'a mut Option<Self::Hd>) -> U,
        G: Fn(Self::Tl) -> U,
    {
        let mut lock = (*self.returning).try_lock();
        if let Ok(ref mut mutex) = lock {
            f(self.singleton.array.clone(), mutex)
        } else {
            panic!("or_return ReturnSingleton: TODO")
        }

        // f(self.singleton.array.clone(), *self.returning)
    }
}


#[derive(Clone, Debug)]
pub enum ReturnOr<'a, T, N, U>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
    U: Elems<N = N>,
{
    Left {
        array: GenericArray<T, N>,
        returning: Arc<Mutex<&'a mut Option<T>>>,
    },
    Right(U),
}

impl<'a, T, N, U> Elems for ReturnOr<'a, T, N, U>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
    U: Elems<N = N>,
{
    type Hd = T;
    type N = N;
    type Tl = U;

    fn or<V, F: Fn(GenericArray<Self::Hd, Self::N>) -> V, G: Fn(Self::Tl) -> V>(&self, f: F, g: G) -> V {
        match self {
            Self::Left { array, .. } => f(array.clone()),
            Self::Right(x) => g(*x),
        }
    }
}

impl<'a, T, N, U> IOElems<'a> for ReturnOr<'a, T, N, U>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
    U: IOElems<'a, N = N>
{
    fn or_return<V, F, G>(&'a self, f: &mut F, g: &mut G) -> V
    where
        F: Fn(GenericArray<Self::Hd, Self::N>, &'a mut Option<Self::Hd>) -> V,
        G: Fn(Self::Tl) -> V,
    {
        match self {
            Self::Left { array, returning } => {
                let mut lock = (*returning).try_lock();
                if let Ok(ref mut mutex) = lock {
                    f(array.clone(), mutex)
                } else {
                    panic!("or_return: TODO")
                }

                // let mut_returning = *returning.lock()?;
                // f(array.clone(), mut_returning)
            },
            Self::Right(x) => g(*x),
        }
    }
}







// + IntoIterator<Item = Elem>
pub trait IsList: Debug {
    type Hd: Elems;
    type Tl: IsList;

    fn is_empty(&self) -> bool;
    fn hd(self) -> Self::Hd;
    fn tl(self) -> Self::Tl;
    fn cons<T: Elems>(self, x: T) -> Cons<T, Self>
    where
        Self: Sized,
    {
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
            array: GenericArray::generate(|_| ()),
        }
    }

    fn tl(self) -> Self::Tl {
        Self {}
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
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

    // // add better errors
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

impl<T, U> IList for Cons<T, U>
where
    T: IElems,
    U: IList,
{
}


pub trait IOList<'a>: IsList {
    type Return: IOElems<'a>;
}

impl<'a, T, U> IOList<'a> for Cons<T, U>
where
    T: IElems,
    U: IOList<'a>,
{
    type Return = U::Return;
}


#[derive(Clone, Debug)]
pub struct ConsOut<'a, T, U>
where
    T: IOElems<'a>,
    U: IList,
{
    a: PhantomData<&'a ()>,
    cons: Cons<T, U>,
}

impl<'a, T, U> IsList for ConsOut<'a, T, U>
where
    T: IOElems<'a>,
    U: IList
{
    type Hd = T;
    type Tl = U;

    fn is_empty(&self) -> bool {
        self.cons.is_empty()
    }

    fn hd(self) -> Self::Hd {
        self.cons.hd()
    }

    fn tl(self) -> Self::Tl {
        self.cons.tl()
    }
}

impl<'a, T, U> IOList<'a> for ConsOut<'a, T, U>
where
    T: IOElems<'a>,
    U: IList,
{
    type Return = T;
}







pub trait IsInstructionT<'a>: std::fmt::Debug {
    type In: IOList<'a>;
    type Error: AnError;

    // fn run(&self, x: Self::In) -> Result<<Self::In as IOList>::Return, Self::Error>;
    fn run(&'a self, x: Self::In) -> Result<(), Self::Error>;
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
// struct Concat<T: AnElem> {
struct Concat<'a> {
    a: PhantomData<&'a ()>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ConcatError {}
impl AnError for ConcatError {}


// // bytes, string, array, object
// impl<'a> IsInstructionT<'a> for Concat<'a> {
//     type In = ConsOut<'a, ReturnOr<'a, Vec<Value>, U2, ReturnSingleton<'a, Map<String, Value>, U2>>, Nil>;
//     type Error = ConcatError;

//     // fn run(&self, x: Self::In) -> Result<<Self::In as IOList>::Return, Self::Error> {
//     fn run(&'a self, x: Self::In) -> Result<(), Self::Error> {
//         let y = x.hd();

//         y.or_return(&mut |z, arc_return| {
//                 let lhs = &z[0];
//                 let rhs = &z[1];
//                 *arc_return = Some(lhs.into_iter().chain(rhs.into_iter()).cloned().collect());
//                 Ok(())
//         },
//                     &mut |z| {
//                 z.or_return(&mut |w, arc_return| {
//                 let lhs = &w[0];
//                 let rhs = &w[1];
//                 *arc_return = Some(lhs.into_iter().chain(rhs.into_iter()).map(|xy| (xy.0.clone(), xy.1.clone())).collect());
//                 Ok(())
//             },
//                     &mut |_w| {
//                 // let lhs = w.singleton.array[0];
//                 // let rhs = w.singleton.array[1];
//                 // **w.returning = Some(lhs.into_iter().chain(rhs.into_iter()).collect());
//                 Ok(())
//             })
//         })

//         // match y {
//         //     Left([z1, z2]) => concat(z1, z2),

//         //     ..
//         // }

//         // let lhs: Or<Vec<Value>, Singleton<Map<String, Value>>> = x.hd()[0].clone();
//         // let rhs: Or<Vec<Value>, Singleton<Map<String, Value>>> = x.hd()[1].clone();
//         // Ok(lhs.into_iter().chain(rhs.into_iter()).collect())
//     }
// }


// // bytes, string, array, object
// impl IsInstructionT for Concat<()> {
//     type In = ConsOut<Or<Vec<Value>, U2, Singleton<Map<String, Value>, U2>>, U2, Singleton<(), U0>, Nil>;
//     type Error = ConcatError;

//     fn run(&self, x: Self::In) -> Result<<Self::In as IOList>::Return, Self::Error> {
//         let y = x.hd();
//         // match y {
//         //     Left([z1, z2]) => concat(z1, z2),

//         //     ..
//         // }

//         // let lhs: Or<Vec<Value>, Singleton<Map<String, Value>>> = x.hd()[0].clone();
//         // let rhs: Or<Vec<Value>, Singleton<Map<String, Value>>> = x.hd()[1].clone();
//         // Ok(lhs.into_iter().chain(rhs.into_iter()).collect())
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


// #[derive(Clone)]
// pub enum ConsOut<T, N, U, V>
// where
//     T: AnElem,
//     N: ArrayLength<T> + Debug,
//     U: Elems<N = N>,
//     V: IList,
// {
//     Left {
//         return_fn: Arc<dyn FnOnce(T) -> Return<(), T>>,
//         hd: Singleton<T, N>,
//         tl: V,
//     },
//     Right {
//         hd: U,
//         tl: V,
//     }
// }

// impl<T, N, U, V> IsList for ConsOut<T, N, U, V>
// where
//     T: AnElem,
//     N: ArrayLength<T> + Debug,
//     U: Elems<N = N>,
//     V: IList,
// {
//     type Hd = Or<T, N, U>;
//     type Tl = V;

//     fn is_empty(&self) -> bool {
//         false
//     }

//     fn hd(self) -> Self::Hd {
//         match self {
//             Self::Left { hd, .. } => Or::Left(hd.t),
//             Self::Right { hd, .. } => Or::Right(hd),
//         }
//     }

//     fn tl(self) -> Self::Tl {
//         match self {
//             Self::Left { tl, .. } => tl,
//             Self::Right { tl, .. } => tl,
//         }
//     }
// }

// impl<T, N, U, V> IOList for ConsOut<T, N, U, V>
// where
//     T: AnElem,
//     N: ArrayLength<T> + Debug,
//     U: Elems<N = N>,
//     V: IList,
// {
//     type Return = T;

//     // fn returning(self) -> Option<Arc<dyn FnOnce(T) -> Return<(), T>>> {
//     //     match self {
//     //         Self::Left { return_fn, .. } => Some(return_fn),
//     //         Self::Right { .. } => None,
//     //     }
//     // }
// }

// impl<T, U> IOList for Cons<T, U>
// where
//     T: Elems,
//     U: IOList,
// {
//     type Return = U::Return;

//     // fn returning(self) -> Option<Arc<dyn FnOnce(U::Return) -> Return<(), U::Return>>> {
//     //     self.tl.returning()
//     // }
// }










// /// return_value is private, but get_return_value can be used to extract it
// #[derive(Clone, Debug, PartialEq, Eq)]
// pub struct Return<T, U: AnElem> {
//     for_instruction: PhantomData<T>,
//     return_value: U,
// }

// impl<T, U: AnElem> Return<T, U> {
//     pub fn get_return_value(self) -> U {
//         self.return_value
//     }
// }







// Cons<Or<U, <Singleton<T>>, Nil>

// ( {U, T} )

// Cons<Returning<Or<U, <Singleton<T>>>, Nil>

// ( {U, T} ) -> {U, T}


// forall x, .. z. IsIn {A, B, C} x, .. => [x, x, y, Bool, y] -> [x, Bool]

// Or < Singleton
// ReturningOr< ReturningSingleton


// <in, out>
// Instruction<in, out>
// Instruction<in, out>
// Instruction<in, out>
// Instruction<in, out>


// [A, B, C]
// Instruction<in, out>
// [A, B, C]



// Or<T, Singleton<()>>

// Or<(), Singleton<()>>

// Or<T, U: SetWithout<T>>

// IsNot<T: AnElem, U: AnElem>

// Dict<dyn IsEq<T, U>> -> Empty

// IsEq<const Ajfijw>
//     type IsEqBool: const bool;




// impl<T, N, U: Elems> AnElem for Or<T, U> {
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

