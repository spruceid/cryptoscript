use crate::elem::{Elem, AnElem, AnElemError, ElemSymbol};
use crate::stack::{Stack, StackError};
use crate::types::{Empty, AnError, Nil};

use std::cmp;
use std::iter::{FromIterator};
use std::marker::PhantomData;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use enumset::EnumSet;
use generic_array::typenum::{U0, U1, U2};
use generic_array::sequence::GenericSequence;
use generic_array::functional::FunctionalSequence;
use generic_array::{arr, GenericArray, GenericArrayIter, ArrayLength};
use serde_json::{Map, Number, Value};

// NEXT:
// - finish migrating instruction implementations from elem to IsInstructionT
// - migrate pop-stack from IsInstruction
// - delete old IsInstruction
// - add typing info as with pop-stack

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
    // TODO: throw error if try_lock fails
    pub fn returning(&self, return_value: T) {
        let mut lock = (*self.return_value).try_lock();
        if let Ok(ref mut mutex) = lock {
            **mutex = Some(return_value)
        } else {
            panic!("returning: TODO")
        }
    }

    // TODO: throw error if try_lock fails
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

// TODO: AnElem: &self -> AllElems<U1>
type AllElems<N> =
    Or<(), N,
    Or<bool, N,
    Or<Number, N,
    Or<Vec<u8>, N,
    Or<String, N,
    Or<Vec<Value>, N,
    Or<Map<String, Value>, N,
    Singleton<Value, N>>>>>>>>;

fn all_elems_untyped<N>(x: &AllElems<N>) -> GenericArray<Elem, N>
where
    N: Debug +
    ArrayLength<()> +
    ArrayLength<bool> +
    ArrayLength<Number> +
    ArrayLength<Vec<u8>> +
    ArrayLength<String> +
    ArrayLength<Vec<Value>> +
    ArrayLength<Map<String, Value>> +
    ArrayLength<Value> +
    ArrayLength<Elem>,
{
    match x {
        Or::Left(array) => {
            array.map(|_x| Elem::Unit)
        },
        Or::Right(Or::Left(array)) => {
            array.map(|&x| Elem::Bool(x))
        },
        Or::Right(Or::Right(Or::Left(array))) => {
            array.map(|x| Elem::Number(x.clone()))
        },
        Or::Right(Or::Right(Or::Right(Or::Left(array)))) => {
            array.map(|x| Elem::Bytes(x.clone()))
        },
        Or::Right(Or::Right(Or::Right(Or::Right(Or::Left(array))))) => {
            array.map(|x| Elem::String(x.clone()))
        },
        Or::Right(Or::Right(Or::Right(Or::Right(Or::Right(Or::Left(array)))))) => {
            array.map(|x| Elem::Array(x.clone()))
        },
        Or::Right(Or::Right(Or::Right(Or::Right(Or::Right(Or::Right(Or::Left(array))))))) => {
            array.map(|x| Elem::Object(x.clone()))
        },
        Or::Right(Or::Right(Or::Right(Or::Right(Or::Right(Or::Right(Or::Right(Singleton { array }))))))) => {
            array.map(|x| Elem::Json(x.clone()))
        },
    }
}





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



// #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
// pub enum Instruction {
//     Restack(Restack),
//
//     CheckLe,
//     CheckLt,
//     CheckEq,
//     Slice,
//     Index,
//     ToJson,
//     UnpackJson(ElemSymbol),
//     StringToBytes,
// }



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
            },
            ReturnOr::Right(ReturnOr::Left { array, returning }) => {
                let lhs = &array[0];
                let rhs = &array[1];
                returning.returning(lhs.into_iter().chain(rhs.into_iter()).cloned().collect());
            },
            ReturnOr::Right(ReturnOr::Right(ReturnSingleton { singleton, returning })) => {
                let lhs = &singleton.array[0];
                let rhs = &singleton.array[1];
                returning.returning(lhs.into_iter().chain(rhs.into_iter()).map(|xy| (xy.0.clone(), xy.1.clone())).collect());
            },
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AssertTrue {}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AssertTrueError {}
impl AnError for AssertTrueError {}

impl IsInstructionT for AssertTrue {
    type In = ConsOut<ReturnSingleton<bool, U1>, Nil>;
    type Error = AssertTrueError;

    fn run(&self, x: Self::In) -> Result<(), Self::Error> {
        let array = x.clone().hd().singleton.array;
        let returning = x.hd().returning;
        if array[0] {
            returning.returning(true);
            Ok(())
        } else {
            Err(AssertTrueError {})
        }
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Push<T: AnElem> {
    push: T,
}

impl<T: AnElem> IsInstructionT for Push<T> {
    type In = ConsOut<ReturnSingleton<T, U0>, Nil>;
    type Error = Empty;

    fn run(&self, x: Self::In) -> Result<(), Self::Error> {
        x.hd().returning.returning(self.push.clone());
        Ok(())
    }
}



#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HashSha256 {}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HashSha256Error {}
impl AnError for HashSha256Error {}

impl IsInstructionT for HashSha256 {
    type In = ConsOut<ReturnSingleton<Vec<u8>, U1>, Nil>;
    type Error = Empty;

    fn run(&self, x: Self::In) -> Result<(), Self::Error> {
        let array = x.clone().hd().singleton.array;
        let returning = x.hd().returning;
        returning.returning(super::sha256(&array[0]));
        Ok(())
    }
}


//#[derive(Clone, Copy, Debug, PartialEq, Eq)]
//struct Slice {}
//#[derive(Clone, Copy, Debug, PartialEq, Eq)]
//struct SliceError {}
//impl AnError for SliceError {}


//#[derive(Clone, Copy, Debug, PartialEq, Eq)]
//struct Index {}
//#[derive(Clone, Copy, Debug, PartialEq, Eq)]
//struct IndexError {}
//impl AnError for IndexError {}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Lookup {}
#[derive(Clone, Debug, PartialEq, Eq)]
struct LookupError {
    key: String,
    map: Map<String, Value>,
}
impl AnError for LookupError {}

impl IsInstructionT for Lookup {
    type In = ConsOut<ReturnSingleton<Value, U0>, Cons<Singleton<String, U1>, Cons<Singleton<Map<String, Value>, U1>, Nil>>>;
    type Error = LookupError;

    fn run(&self, x: Self::In) -> Result<(), Self::Error> {
        let returning = x.clone().hd().returning;
        let key = &x.clone().tl().hd().array[0];
        let map = &x.tl().tl().hd().array[0];
        returning.returning(map.get(key)
           .ok_or_else(|| LookupError {
               key: key.clone(),
               map: map.clone(),
           })?.clone());
        Ok(())
    }
}


// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
// struct UnpackJson<T: AnElem> {
//     t: PhantomData<T>,
// }
// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
// struct UnpackJsonError {}
// impl AnError for UnpackJsonError {}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct StringToBytes {}
// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
// struct StringToBytesError {}
// impl AnError for StringToBytesError {}

impl IsInstructionT for StringToBytes {
    type In = ConsOut<ReturnSingleton<Vec<u8>, U0>, Cons<Singleton<String, U1>, Nil>>;
    type Error = Empty;

    fn run(&self, x: Self::In) -> Result<(), Self::Error> {
        let returning = x.clone().hd().returning;
        let in_str = &x.clone().tl().hd().array[0];
        returning.returning(in_str.clone().into_bytes());
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CheckLe {}
#[derive(Clone, Debug, PartialEq, Eq)]
struct CheckLeError {
    lhs: Elem,
    rhs: Elem,
}
impl AnError for CheckLeError {}

impl IsInstructionT for CheckLe {
    type In = ConsOut<ReturnSingleton<bool, U0>, Cons<AllElems<U2>, Nil>>;
    type Error = CheckLeError;

    fn run(&self, x: Self::In) -> Result<(), Self::Error> {
        let returning = x.clone().hd().returning;
        let y = &x.clone().tl().hd();
        let array = all_elems_untyped(y);
        let lhs = array[0].clone();
        let rhs = array[1].clone();
        let cmp_result = lhs.partial_cmp(&rhs)
            .ok_or_else(|| CheckLeError {
                lhs: lhs,
                rhs: rhs
        })?;
        let result = match cmp_result {
            cmp::Ordering::Less => true,
            cmp::Ordering::Equal => true,
            cmp::Ordering::Greater => false,
        };
        returning.returning(result);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CheckLt {}
#[derive(Clone, Debug, PartialEq, Eq)]
struct CheckLtError {
    lhs: Elem,
    rhs: Elem,
}
impl AnError for CheckLtError {}

impl IsInstructionT for CheckLt {
    type In = ConsOut<ReturnSingleton<bool, U0>, Cons<AllElems<U2>, Nil>>;
    type Error = CheckLtError;

    fn run(&self, x: Self::In) -> Result<(), Self::Error> {
        let returning = x.clone().hd().returning;
        let y = &x.clone().tl().hd();
        let array = all_elems_untyped(y);
        let lhs = array[0].clone();
        let rhs = array[1].clone();
        let cmp_result = lhs.partial_cmp(&rhs)
            .ok_or_else(|| CheckLtError {
                lhs: lhs,
                rhs: rhs
        })?;
        let result = match cmp_result {
            cmp::Ordering::Less => true,
            _ => false,
        };
        returning.returning(result);
        Ok(())
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CheckEq {}
#[derive(Clone, Debug, PartialEq, Eq)]
struct CheckEqError {
    lhs: Elem,
    rhs: Elem,
}
impl AnError for CheckEqError {}

impl IsInstructionT for CheckEq {
    type In = ConsOut<ReturnSingleton<bool, U0>, Cons<AllElems<U2>, Nil>>;
    type Error = CheckEqError;

    fn run(&self, x: Self::In) -> Result<(), Self::Error> {
        let returning = x.clone().hd().returning;
        let y = &x.clone().tl().hd();
        let array = all_elems_untyped(y);
        let lhs = array[0].clone();
        let rhs = array[1].clone();
        let cmp_result = lhs.partial_cmp(&rhs)
            .ok_or_else(|| CheckEqError {
                lhs: lhs,
                rhs: rhs
        })?;
        let result = match cmp_result {
            cmp::Ordering::Equal => true,
            _ => false,
        };
        returning.returning(result);
        Ok(())
    }
}

    // fn slice_generic<T: Clone + IntoIterator +
    //   std::iter::FromIterator<<T as std::iter::IntoIterator>::Item>>(offset: Number,
    //                                                                  length: Number,
    //                                                                  iterable: T,
    //                                                                  elem_symbol: ElemSymbol) ->
    //     Result<T, ElemError> {
    //     let u_offset = offset.as_u64()
    //         .ok_or_else(|| ElemError::SliceOffsetNotU64(offset.clone()))
    //         .and_then(|x| usize::try_from(x).map_err(|_| ElemError::SliceOverflow { offset: offset.clone(), length: length.clone() }))?;
    //     let u_length = length.as_u64()
    //         .ok_or_else(|| ElemError::SliceLengthNotU64(length.clone()))
    //         .and_then(|x| usize::try_from(x).map_err(|_| ElemError::SliceOverflow { offset: offset.clone(), length: length.clone() }))?;
    //     let u_offset_plus_length = u_offset.checked_add(u_length)
    //         .ok_or_else(|| ElemError::SliceOverflow { offset: offset.clone(), length: length.clone() })?;
    //     if iterable.clone().into_iter().count() < u_offset_plus_length {
    //         Err(ElemError::SliceTooShort {
    //             offset: u_offset,
    //             length: u_length,
    //             iterable: From::from(elem_symbol),
    //         })
    //     } else {
    //         Ok(iterable.into_iter().skip(u_offset).take(u_length).collect())
    //     }
    // }

    // pub fn slice(maybe_offset: Self, maybe_length: Self, maybe_iterable: Self) -> Result<Self, ElemError> {
    //     match (maybe_offset, maybe_length, maybe_iterable) {
    //         (Self::Number(offset), Self::Number(length), Self::Bytes(iterator)) =>
    //             Ok(Self::Bytes(Self::slice_generic(offset, length, iterator, ElemSymbol::Bytes)?)),
    //         (Self::Number(offset), Self::Number(length), Self::String(iterator)) => {
    //             let iterator_vec = Vec::from(iterator.clone());
    //             Ok(Self::String(String::from_utf8(Self::slice_generic(offset.clone(), length.clone(), iterator_vec, ElemSymbol::String)?)
    //                     .map_err(|_| ElemError::SliceInvalidUTF8 { offset: offset, length: length, iterator: iterator })?))
    //             },
    //         (Self::Number(offset), Self::Number(length), Self::Array(iterator)) =>
    //             Ok(Self::Array(Self::slice_generic(offset, length, iterator, ElemSymbol::Number)?)),
    //         (Self::Number(offset), Self::Number(length), Self::Object(iterator)) =>
    //             Ok(Self::Object(Self::slice_generic(offset, length, iterator, ElemSymbol::Object)?)),
    //         (maybe_not_offset, maybe_not_length, maybe_not_iterable) => {
    //             Err(ElemError::SliceUnsupportedTypes {
    //                 maybe_not_offset: maybe_not_offset.symbol_str(),
    //                 maybe_not_length: maybe_not_length.symbol_str(),
    //                 maybe_not_iterable: maybe_not_iterable.symbol_str(),
    //             })
    //         }
    //     }
    // }

    // fn index_generic<T: Clone + IntoIterator +
    //     std::iter::FromIterator<<T as std::iter::IntoIterator>::Item>>(index: Number,
    //                                                                    iterable: T,
    //                                                                    elem_symbol: ElemSymbol) ->
    //   Result<<T as std::iter::IntoIterator>::Item, ElemError> {
    //     let u_index: usize = index.as_u64()
    //         .ok_or_else(|| ElemError::IndexNotU64(index.clone()))
    //         .and_then(|x| usize::try_from(x).map_err(|_| ElemError::IndexOverflow(index.clone())))?;
    //     if iterable.clone().into_iter().count() <= u_index {
    //         return Err(ElemError::IndexTooShort {
    //             index: u_index,
    //             iterable: From::from(elem_symbol),
    //         })
    //     } else {
    //         match iterable.into_iter().skip(u_index).next() {
    //             None => Err(ElemError::IndexTooShort { index: u_index, iterable: From::from(elem_symbol) }),
    //             Some(x) => Ok(x),
    //         }
    //     }
    // }

    // pub fn index(self, maybe_iterable: Self) -> Result<Self, ElemError> {
    //     match (self, maybe_iterable) {
    //         // (Self::Number(index), Self::Bytes(iterator)) =>
    //         //     Ok(Self::Bytes(vec![Self::index_generic(index, iterator, ElemSymbol::Bytes)?])),
    //         (Self::Number(index), Self::Array(iterator)) =>
    //             Ok(Self::Json(Self::index_generic(index, iterator, ElemSymbol::Json)?)),
    //         (Self::Number(index), Self::Object(iterator)) =>
    //             Ok(Self::Json(Self::index_generic(index, iterator, ElemSymbol::Object)?.1)),
    //         (maybe_not_index, maybe_not_iterable) => {
    //             Err(ElemError::IndexUnsupportedTypes {
    //                 maybe_not_index: maybe_not_index.symbol_str(),
    //                 maybe_not_iterable: maybe_not_iterable.symbol_str(),
    //             })
    //         }
    //     }
    // }

    // pub fn to_json(self) -> Result<Self, ElemError> {
    //     Ok(Self::Json(serde_json::to_value(self)?))
    // }

    // pub fn unpack_json(self, elem_symbol: ElemSymbol) -> Result<Self, ElemError> {
    //     match (self, elem_symbol) {
    //         (Self::Json(serde_json::Value::Null), ElemSymbol::Unit) => Ok(Self::Unit),
    //         (Self::Json(serde_json::Value::Bool(x)), ElemSymbol::Bool) => Ok(Self::Bool(x)),
    //         (Self::Json(serde_json::Value::Number(x)), ElemSymbol::Number) => Ok(Self::Number(x)),
    //         (Self::Json(serde_json::Value::String(x)), ElemSymbol::String) => Ok(Self::String(x)),
    //         (Self::Json(serde_json::Value::Array(x)), ElemSymbol::Array) => Ok(Self::Array(x)),
    //         (Self::Json(serde_json::Value::Object(x)), ElemSymbol::Object) => Ok(Self::Object(x)),
    //         (Self::Json(json), elem_symbol) => Err(ElemError::UnpackJsonUnsupportedSymbol {
    //           json: json,
    //           elem_symbol: From::from(elem_symbol),
    //         }),
    //         (non_json, _) => Err(ElemError::UnpackJsonUnexpectedType {
    //               non_json: non_json.symbol_str(),
    //               elem_symbol: From::from(elem_symbol),
    //         }),
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

