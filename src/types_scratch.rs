use crate::elem::{Elem, AnElem, AnElemError, ElemSymbol};
use crate::stack::{Stack, StackError};
use crate::types::{Empty, AnError, Nil};

use std::cmp;
use std::convert::TryFrom;
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::string::FromUtf8Error;

use enumset::EnumSet;
use generic_array::typenum::{U0, U1, U2};
use generic_array::sequence::GenericSequence;
use generic_array::functional::FunctionalSequence;
use generic_array::{arr, GenericArray, GenericArrayIter, ArrayLength};
use serde_json::{Map, Number, Value};
use thiserror::Error;

// use generic_array::typenum::{B1};
// use typenum::marker_traits::Unsigned;
// use typenum::type_operators::IsLess;

// NEXT:
// - migrate pop from ElemList
//     fn pop(x: PhantomData<Self>, stack: &mut Stack) -> Result<Self, StackError>;

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
    type IO: IOList;
    type Error: AnError;

    fn run(&self, x: Self::IO) -> Result<(), Self::Error>;
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Concat {}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConcatError {}
impl AnError for ConcatError {}

// TODO: add string!
// (Self::String(x), Self::String(y)) => {
//     Ok(Self::String(String::from_utf8(Self::concat_generic(Vec::from(x.clone()), Vec::from(y.clone())))
//                     .map_err(|_| ElemError::ConcatInvalidUTF8 { lhs: x, rhs: y })?))
// },
//
// bytes, array, object
impl IsInstructionT for Concat {
    type IO = ConsOut<ReturnOr<Vec<u8>,             U2,
                      ReturnOr<Vec<Value>,          U2,
               ReturnSingleton<Map<String, Value>,  U2>>>, Nil>;
    type Error = ConcatError;

    fn run(&self, x: Self::IO) -> Result<(), Self::Error> {
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
pub struct AssertTrue {}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AssertTrueError {}
impl AnError for AssertTrueError {}

impl IsInstructionT for AssertTrue {
    type IO = ConsOut<ReturnSingleton<bool, U1>, Nil>;
    type Error = AssertTrueError;

    fn run(&self, x: Self::IO) -> Result<(), Self::Error> {
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
pub struct Push<T: AnElem> {
    push: T,
}

impl<T: AnElem> IsInstructionT for Push<T> {
    type IO = ConsOut<ReturnSingleton<T, U0>, Nil>;
    type Error = Empty;

    fn run(&self, x: Self::IO) -> Result<(), Self::Error> {
        x.hd().returning.returning(self.push.clone());
        Ok(())
    }
}



#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HashSha256 {}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HashSha256Error {}
impl AnError for HashSha256Error {}

impl IsInstructionT for HashSha256 {
    type IO = ConsOut<ReturnSingleton<Vec<u8>, U1>, Nil>;
    type Error = Empty;

    fn run(&self, x: Self::IO) -> Result<(), Self::Error> {
        let array = x.clone().hd().singleton.array;
        let returning = x.hd().returning;
        returning.returning(super::sha256(&array[0]));
        Ok(())
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Slice {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SliceError {
    OffsetNotU64(Number),

    LengthNotU64(Number),

    Overflow {
        offset: Number,
        length: Number,
    },

    TooShort {
        offset: usize,
        length: usize,
        iterable: String,
    },

    FromUtf8Error(FromUtf8Error),
}

impl From<FromUtf8Error> for SliceError {
    fn from(error: FromUtf8Error) -> Self {
        Self::FromUtf8Error(error)
    }
}

impl AnError for SliceError {}

// bytes, string, array, object
impl IsInstructionT for Slice {
    type IO = ConsOut<ReturnOr<Vec<u8>,             U1,
                      ReturnOr<String,              U1,
                      ReturnOr<Vec<Value>,          U1,
               ReturnSingleton<Map<String, Value>,  U1>>>>,
                Cons<Singleton<Number,              U2>, Nil>>;
    type Error = SliceError;

    fn run(&self, x: Self::IO) -> Result<(), Self::Error> {
        let y = x.clone().hd();
        let offset_length = x.clone().tl().hd().array;
        let offset = &offset_length[0];
        let length = &offset_length[1];
        let u_offset = offset.as_u64()
            .ok_or_else(|| SliceError::OffsetNotU64(offset.clone()))
            .and_then(|x| usize::try_from(x).map_err(|_| SliceError::Overflow { offset: offset.clone(), length: length.clone() }))?;
        let u_length = length.as_u64()
            .ok_or_else(|| SliceError::LengthNotU64(length.clone()))
            .and_then(|x| usize::try_from(x).map_err(|_| SliceError::Overflow { offset: offset.clone(), length: length.clone() }))?;
        let u_offset_plus_length = u_offset.checked_add(u_length)
            .ok_or_else(|| SliceError::Overflow { offset: offset.clone(), length: length.clone() })?;
        match y.clone() {
            ReturnOr::Left { array, returning } => {
                let iterable = &array[0];
                if iterable.clone().into_iter().count() < u_offset_plus_length {
                    Err(())
                } else {
                    returning.returning(iterable.into_iter().skip(u_offset).take(u_length).copied().collect());
                    Ok(())
                }
            },
            ReturnOr::Right(ReturnOr::Left { array, returning }) => {
                let iterable = &array[0];
                if iterable.len() < u_offset_plus_length {
                    Err(())
                } else {
                    returning.returning(String::from_utf8(Vec::from(iterable.clone()).into_iter().skip(u_offset).take(u_length).collect())?);
                    Ok(())
                }
            },
            ReturnOr::Right(ReturnOr::Right(ReturnOr::Left { array, returning })) => {
                let iterable = &array[0];
                if iterable.clone().into_iter().count() < u_offset_plus_length {
                    Err(())
                } else {
                    returning.returning(iterable.into_iter().skip(u_offset).take(u_length).cloned().collect());
                    Ok(())
                }
            },
            ReturnOr::Right(ReturnOr::Right(ReturnOr::Right(ReturnSingleton { singleton: Singleton { array }, returning }))) => {
                let iterable = &array[0];
                if iterable.clone().into_iter().count() < u_offset_plus_length {
                    Err(())
                } else {
                    returning.returning(iterable.into_iter().skip(u_offset).take(u_length).map(|xy| (xy.0.clone(), xy.1.clone())).collect());
                    Ok(())
                }
            },
        }.map_err(|_e| {
            SliceError::TooShort {
                offset: u_offset,
                length: u_length,
                // TODO: better error
                iterable: format!("{:?}", y),
            }
        })
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Index {}
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum IndexError {
    #[error("Index: index not valid u64: {0:?}")]
    IndexNotU64(Number),

    #[error("Index: index not valid usize: {0:?}")]
    Overflow(Number),

    #[error("Index: iterable: {iterable:?}\nis too short for index: {index:?}")]
    TooShort {
        index: usize,
        iterable: String,
    },
}
impl AnError for IndexError {}

// bytes, array, object
impl IsInstructionT for Index {
    type IO = ConsOut<ReturnSingleton<Value,                U0>,
                              Cons<Or<Vec<Value>,           U2,
                            Singleton<Map<String, Value>,   U2>>,
                       Cons<Singleton<Number,               U1>, Nil>>>;
    type Error = IndexError;

    fn run(&self, x: Self::IO) -> Result<(), Self::Error> {
        let returning = x.clone().hd().returning;
        let y = x.clone().tl().hd();
        let index = &x.clone().tl().tl().hd().array[0];
        let u_index = index.as_u64()
            .ok_or_else(|| IndexError::IndexNotU64(index.clone()))
            .and_then(|x| usize::try_from(x).map_err(|_| IndexError::Overflow(index.clone())))?;
        let result = match y.clone() {
            Or::Left(array) => {
                array[0]
                    .clone()
                    .into_iter()
                    .skip(u_index)
                    .next()
            },
            Or::Right(Singleton { array }) => {
                array[0]
                    .clone()
                    .into_iter()
                    .skip(u_index)
                    .next()
                    .map(|(_x, y)| y)
            },
        }.ok_or_else(|| {
            IndexError::TooShort {
                index: u_index,
                // TODO: better error
                iterable: format!("{:?}", y),
            }
        })?;
        returning.returning(result);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToJson {}
#[derive(Clone, Debug)]
pub struct ToJsonError {
    input: Elem,
    error: Arc<serde_json::Error>,
}
impl AnError for ToJsonError {}

impl IsInstructionT for ToJson {
    type IO = ConsOut<ReturnSingleton<Value, U0>, Cons<AllElems<U1>, Nil>>;
    type Error = ToJsonError;

    fn run(&self, x: Self::IO) -> Result<(), Self::Error> {
        let returning = x.clone().hd().returning;
        let y = &x.clone().tl().hd();
        let array = all_elems_untyped(y);
        let z = array[0].clone();
        returning.returning(serde_json::to_value(z.clone())
                            .map_err(move |e| ToJsonError {
                                input: z,
                                error: Arc::new(e),
        })?);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lookup {}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LookupError {
    key: String,
    map: Map<String, Value>,
}
impl AnError for LookupError {}

impl IsInstructionT for Lookup {
    type IO = ConsOut<ReturnSingleton<Value, U0>, Cons<Singleton<String, U1>, Cons<Singleton<Map<String, Value>, U1>, Nil>>>;
    type Error = LookupError;

    fn run(&self, x: Self::IO) -> Result<(), Self::Error> {
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


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct UnpackJson<T: AnElem> {
    t: PhantomData<T>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct UnpackJsonError {}
impl AnError for UnpackJsonError {}

// TODO: implement for rest of types
trait AJsonElem: AnElem {
    fn to_value(&self) -> Value;
    fn from_value(t: PhantomData<Self>, x: Value) -> Option<Self>;
}

impl AJsonElem for () {
    fn to_value(&self) -> Value {
        Value::Null
    }

    fn from_value(_t: PhantomData<Self>, x: Value) -> Option<Self> {
        match x {
            Value::Null => Some(()),
            _ => None,
        }
    }
}

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

impl<T: AJsonElem> IsInstructionT for UnpackJson<T> {
    type IO = ConsOut<ReturnSingleton<T, U0>,
                       Cons<Singleton<Value, U1>, Nil>>;
    type Error = UnpackJsonError;

    fn run(&self, x: Self::IO) -> Result<(), Self::Error> {
        let returning = x.clone().hd().returning;
        let json = &x.clone().tl().hd().array[0];
        let result =
            AJsonElem::from_value(PhantomData::<T>, json.clone())
            .ok_or_else(|| UnpackJsonError {})?;
        returning.returning(result);
        Ok(())
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StringToBytes {}

impl IsInstructionT for StringToBytes {
    type IO = ConsOut<ReturnSingleton<Vec<u8>, U0>, Cons<Singleton<String, U1>, Nil>>;
    type Error = Empty;

    fn run(&self, x: Self::IO) -> Result<(), Self::Error> {
        let returning = x.clone().hd().returning;
        let in_str = &x.clone().tl().hd().array[0];
        returning.returning(in_str.clone().into_bytes());
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CheckLe {}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckLeError {
    lhs: Elem,
    rhs: Elem,
}
impl AnError for CheckLeError {}

impl IsInstructionT for CheckLe {
    type IO = ConsOut<ReturnSingleton<bool, U0>, Cons<AllElems<U2>, Nil>>;
    type Error = CheckLeError;

    fn run(&self, x: Self::IO) -> Result<(), Self::Error> {
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
pub struct CheckLt {}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckLtError {
    lhs: Elem,
    rhs: Elem,
}
impl AnError for CheckLtError {}

impl IsInstructionT for CheckLt {
    type IO = ConsOut<ReturnSingleton<bool, U0>, Cons<AllElems<U2>, Nil>>;
    type Error = CheckLtError;

    fn run(&self, x: Self::IO) -> Result<(), Self::Error> {
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
pub struct CheckEq {}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckEqError {
    lhs: Elem,
    rhs: Elem,
}
impl AnError for CheckEqError {}

impl IsInstructionT for CheckEq {
    type IO = ConsOut<ReturnSingleton<bool, U0>, Cons<AllElems<U2>, Nil>>;
    type Error = CheckEqError;

    fn run(&self, x: Self::IO) -> Result<(), Self::Error> {
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

