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
    fn or<T, F: Fn(&GenericArray<Self::Hd, Self::N>) -> T, G: Fn(&Self::Tl) -> T>(&self, f: F, g: G) -> T;

    // fn elem_symbols(t: PhantomData<Self>) -> EnumSet<ElemSymbol>;
    // fn to_elems(self) -> Elem;
    // fn from_elems(t: PhantomData<Self>, x: &mut Stack) -> Result<Self, ElemsError>;
}

pub trait IElems: Elems {}


#[derive(Clone, Debug)]
pub struct Return<T: AnElem> {
    return_value: Arc<Mutex<Option<T>>>,
}

impl<T: AnElem> Return<T> {
    pub fn returning(&self, return_value: T) {
        let mut lock = (*self.return_value).try_lock();
        if let Ok(ref mut mutex) = lock {
            **mutex = Some(return_value)
        } else {
            panic!("returning: TODO")
        }
    }

    pub fn returned(&self) -> Option<T> {
        let mut lock = (*self.return_value).try_lock();
        if let Ok(ref mut mutex) = lock {
            (**mutex).clone()
        } else {
            panic!("returned: TODO")
        }
    }
}

pub trait IOElems: Elems {
    fn or_return<T, F, G>(&self, f: F, g: G) -> T
        where
            F: Fn(&GenericArray<Self::Hd, Self::N>, &Return<Self::Hd>) -> T,
            G: Fn(&Self::Tl) -> T;
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
    fn or<V, F: Fn(&GenericArray<Self::Hd, Self::N>) -> V, G: Fn(&Self::Tl) -> V>(&self, f: F, g: G) -> V {
        match self {
            Self::Left(x) => f(x),
            Self::Right(x) => g(x),
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
pub struct ReturnSingleton<T, N>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
{
    singleton: Singleton<T, N>,
    returning: Return<T>,
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
}


#[derive(Clone, Debug)]
pub enum ReturnOr<T, N, U>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
    U: Elems<N = N>,
{
    Left {
        array: GenericArray<T, N>,
        returning: Return<T>,
    },
    Right(U),
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


pub trait IOList: IsList {
    type Return: IOElems;
}

impl<T, U> IOList for Cons<T, U>
where
    T: IElems,
    U: IOList,
{
    type Return = U::Return;
}


#[derive(Clone, Debug)]
pub struct ConsOut<T, U>
where
    T: IOElems,
    U: IList,
{
    cons: Cons<T, U>,
}

impl<T, U> IsList for ConsOut<T, U>
where
    T: IOElems,
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

impl<T, U> IOList for ConsOut<T, U>
where
    T: IOElems,
    U: IList,
{
    type Return = T;
}







pub trait IsInstructionT: std::fmt::Debug {
    type In: IOList;
    type Error: AnError;

    fn run(&self, x: Self::In) -> Result<(), Self::Error>;
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Concat {
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ConcatError {}
impl AnError for ConcatError {}

// bytes, array, object
impl IsInstructionT for Concat {
    type In = ConsOut<ReturnOr<Vec<Value>,          U2,
                      ReturnOr<Vec<Value>,          U2,
               ReturnSingleton<Map<String, Value>,  U2>>>, Nil>;
    type Error = ConcatError;

    fn run(&self, x: Self::In) -> Result<(), Self::Error> {
        let y = x.hd();
        match y {
            ReturnOr::Left { array, returning } => {
                let lhs = &array[0];
                let rhs = &array[1];
                returning.returning(lhs.into_iter().chain(rhs.into_iter()).cloned().collect());
                Ok(())
            },
            ReturnOr::Right(ReturnOr::Left { array, returning }) => {
                let lhs = &array[0];
                let rhs = &array[1];
                returning.returning(lhs.into_iter().chain(rhs.into_iter()).cloned().collect());
                Ok(())
            },
            ReturnOr::Right(ReturnOr::Right(ReturnSingleton { singleton, returning })) => {
                let lhs = &singleton.array[0];
                let rhs = &singleton.array[1];
                returning.returning(lhs.into_iter().chain(rhs.into_iter()).map(|xy| (xy.0.clone(), xy.1.clone())).collect());
                Ok(())
            },
        }
    }
}

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

