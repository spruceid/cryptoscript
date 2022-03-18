use crate::elem::{Elem, ElemType, ElemTypeError, ElemSymbol, AnElem};
use crate::stack::{Stack, StackError};
use crate::restack::{Restack, RestackError};
use crate::types::{Context, ContextError, Type, Empty, AnError, Nil};

use std::cmp;
use std::convert::TryFrom;
// use std::iter::FromIterator;
use std::marker::PhantomData;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::string::FromUtf8Error;

use enumset::EnumSet;
use generic_array::functional::FunctionalSequence;
use generic_array::sequence::GenericSequence;
use generic_array::typenum::{U0, U1, U2};
use generic_array::{GenericArray, GenericArrayIter, ArrayLength};
use serde_json::{Map, Number, Value};
use thiserror::Error;
use typenum::marker_traits::Unsigned;

// use generic_array::typenum::{B1};
// use typenum::marker_traits::Unsigned;
// use typenum::type_operators::IsLess;

// NEXT:
// - delete old IsInstruction
//
// - add typing info as with pop-stack
//  + get typing up to parity
//  + add special-case unifier for restack + IsInstructionT for testing?
//
// - random type -> ~random inhabitant of the type
// - random typed program!?

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Singleton<T, N>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
{
    array: GenericArray<T, N>,
}

impl<T, N> IntoIterator for Singleton<T, N>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
{
    type Item = Elem;
    type IntoIter = std::iter::Map<GenericArrayIter<T, N>, fn(T) -> Elem>;

    fn into_iter(self) -> Self::IntoIter {
        self.array.into_iter().map(AnElem::to_elem)
    }
}

#[derive(Clone, Debug, Error)]
pub enum ElemsPopError {
    #[error("Elems::pop singleton: tried to pop an Elem that was not found: {error:?}")]
    PopSingleton {
        error: StackError,
    },

    #[error("Elems::pop: tried to pop a set of Elem's that were not found: {hd_error:?}\n{tl_errors:?}")]
    Pop {
        hd_error: Arc<Self>,
        tl_errors: Arc<Self>,
    },

    // TODO: add detail
    #[error("Elems::pop: generic_array internal error\n\nelem_set:\n{elem_set:?}\n\nvec:\n{vec:?}\n\nsize:\n{size}")]
    GenericArray {
        elem_set: EnumSet<ElemSymbol>,
        vec: Vec<Elem>,
        size: usize,
    },

    #[error("IsList::pop (Cons, Hd): tried to pop a set of Elem's that were not found:\n{stack_type:?}\n{elem_set:?}\n{stack:?}\n\nerror:\n{error}")]
    IsListHd {
        stack_type: Result<Vec<ElemType>, Arc<Self>>,
        elem_set: Result<ElemType, Arc<Self>>,
        stack: Stack,
        error: Arc<Self>,
    },

    #[error("IsList::pop (Cons, Tl): tried to pop a set of Elem's that were not found:\n{stack_type:?}\n{stack:?}\n\nerror:\n{error}")]
    IsListTl {
        stack_type: Result<Vec<ElemType>, Arc<Self>>,
        stack: Stack,
        error: Arc<Self>,
    },

    #[error("Elems::elem_type (Or): Set includes repeated type: {0:?}")]
    ElemTypeError(ElemTypeError),

    #[error("<ReturnOr as IOElems>::type_of(): ContextError when adding type: {0:?}")]
    ReturnOrContextError(ContextError),
}

impl From<StackError> for ElemsPopError {
    fn from(error: StackError) -> Self {
        Self::PopSingleton {
            error: error,
        }
    }
}

pub trait Elems: Clone + Debug + IntoIterator<Item = Elem> {
    type Hd: AnElem;
    type N: ArrayLength<Self::Hd>;
    type Tl: Elems<N = Self::N>;

    // fn left(s: PhantomData<Self>, x: GenericArray<Self::Hd, Self::N>) -> Self;
    // fn right(s: PhantomData<Self>, x: Self::Tl) -> Self;
    fn or<T, F: Fn(&GenericArray<Self::Hd, Self::N>) -> T, G: Fn(&Self::Tl) -> T>(&self, f: F, g: G) -> T;

    // fn to_elems(self) -> Elem;
    // fn from_elems(t: PhantomData<Self>, x: &mut Stack) -> Result<Self, ElemsError>;

    fn pop(_x: PhantomData<Self>, stack: &mut Stack) -> Result<Self, ElemsPopError>
    where
        Self: Sized;

    fn elem_type(t: PhantomData<Self>) -> Result<ElemType, ElemsPopError>;
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

    // TODO: rename to 'returned' to match Return<T>
    fn returning(&self) -> Option<Elem>;

    fn type_of(t: PhantomData<Self>) -> Result<Type, ElemsPopError>;
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

    fn pop(_x: PhantomData<Self>, stack: &mut Stack) -> Result<Self, ElemsPopError>
    where
        Self: Sized,
    {
        let vec = (0..<N as Unsigned>::to_usize()).map(|_array_ix| {
            stack
                .pop_elem(PhantomData::<T>)
                .map_err(|e| <ElemsPopError as From<StackError>>::from(e))
        }).collect::<Result<Vec<T>, ElemsPopError>>()?;
        let array = GenericArray::from_exact_iter(vec.clone()).ok_or_else(|| {
            ElemsPopError::GenericArray {
                elem_set: AnElem::elem_symbol(PhantomData::<T>),
                vec: vec.into_iter().map(|x| x.to_elem()).collect(),
                size: <N as Unsigned>::to_usize(),
            }
        })?;
        Ok(Singleton {
            array: array,
        })
    }

    // TODO: add info
    fn elem_type(_t: PhantomData<Self>) -> Result<ElemType, ElemsPopError> {
        Ok(ElemType {
            type_set: AnElem::elem_symbol(PhantomData::<T>),
            info: vec![],
        })
    }
}


// // TODO: relocate LineNo, ArgumentIndex, Location
// #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
// pub struct LineNo {
//     pub line_no: usize,
// }

// impl From<usize> for LineNo {
//     fn from(line_no: usize) -> Self {
//         LineNo {
//             line_no: line_no,
//         }
//     }
// }

// pub type ArgumentIndex = usize;

// #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
// pub struct Location {
//     line_no: LineNo,
//     argument_index: ArgumentIndex,
//     is_input: bool,
// }

// impl LineNo {
//     pub fn in_at(&self, argument_index: usize) -> Location {
//         Location {
//             line_no: *self,
//             argument_index: argument_index,
//             is_input: true,
//         }
//     }

//     pub fn out_at(&self, argument_index: usize) -> Location {
//         Location {
//             line_no: *self,
//             argument_index: argument_index,
//             is_input: false,
//         }
//     }
// }


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

// #[derive(Clone, Debug, PartialEq, Eq)]
pub enum IterOr<T, N, U>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
    U: Elems<N = N>,
{
    Left(<Singleton<T, N> as IntoIterator>::IntoIter),
    Right(<U as IntoIterator>::IntoIter),
}

impl<T, N, U> Iterator for IterOr<T, N, U>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
    U: Elems<N = N>,
{
    type Item = Elem;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Left(x) => x.next(),
            Self::Right(x) => x.next(),
        }
    }
}

impl<T, N, U> IntoIterator for Or<T, N, U>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
    U: Elems<N = N>,
{
    type Item = Elem;
    type IntoIter = IterOr<T, N, U>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Left(array) => IterOr::Left(
                Singleton {
                    array: array,
                }.into_iter()
            ),
            Self::Right(xs) => IterOr::Right(xs.into_iter()),
        }
    }
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

    fn pop(_x: PhantomData<Self>, stack: &mut Stack) -> Result<Self, ElemsPopError>
    where
        Self: Sized,
    {
        match <Singleton<T, N> as Elems>::pop(PhantomData, stack) {
            Ok(Singleton { array }) => Ok(Self::Left(array)),
            Err(hd_error) => {
                Elems::pop(PhantomData::<U>, stack)
                    .map(|x| Self::Right(x))
                    .map_err(|tl_errors| {
                        ElemsPopError::Pop {
                            hd_error: Arc::new(hd_error),
                            tl_errors: Arc::new(tl_errors),
                        }
                    })
            },
        }
    }

    // TODO: add info
    fn elem_type(_t: PhantomData<Self>) -> Result<ElemType, ElemsPopError> {
        let elem_type_hd = ElemType {
            type_set: AnElem::elem_symbol(PhantomData::<T>),
            info: vec![],
        };
        let elem_type_tl = Elems::elem_type(PhantomData::<U>)?;
        elem_type_hd.unify(elem_type_tl)
            .map_err(|e| ElemsPopError::ElemTypeError(e))
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
            returning: Return {
                return_value: Arc::new(Mutex::new(None)),
            },
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
            context: context,
            i_type: (1..num_inputs).into_iter().map(|_| type_id).collect(),
            o_type: vec![type_id],
        })
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
                        returning: Return {
                            return_value: Arc::new(Mutex::new(None)),
                        },
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
        let mut type_tl = IOElems::type_of(PhantomData::<U>)?;
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







pub trait IsList: Clone + Debug + IntoIterator<Item = Elem> {
    type Hd: Elems;
    type Tl: IsList;

    fn empty_list() -> Option<Self> where Self: Sized;
    fn cons_list(x: Self::Hd, xs: Self::Tl) -> Self;

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

    fn elem_type_vec(t: PhantomData<Self>) -> Result<Vec<ElemType>, ElemsPopError>;

    fn pop(_x: PhantomData<Self>, stack: &mut Stack) -> Result<Self, ElemsPopError>
    where
        Self: Sized,
    {
        match <Self as IsList>::empty_list() {
            Some(x) => Ok(x),
            None => {
                let original_stack = stack.clone();
                let x = <Self::Hd as Elems>::pop(PhantomData, stack).map_err(|e| ElemsPopError::IsListHd {
                    stack_type: IsList::elem_type_vec(PhantomData::<Self>).map_err(|e| Arc::new(e)),
                    elem_set: Elems::elem_type(PhantomData::<Self::Hd>).map_err(|e| Arc::new(e)),
                    stack: original_stack.clone(),
                    error: Arc::new(e),
                })?;
                let xs = <Self::Tl as IsList>::pop(PhantomData, stack).map_err(|e| ElemsPopError::IsListTl {
                    stack_type: IsList::elem_type_vec(PhantomData::<Self>).map_err(|e| Arc::new(e)),
                    stack: original_stack.clone(),
                    error: Arc::new(e),
                })?;
                Ok(<Self as IsList>::cons_list(x, xs))
            }
        }
    }
}

impl IsList for Nil {
    type Hd = Singleton<(), U0>;
    type Tl = Nil;

    fn empty_list() -> Option<Self> where Self: Sized {
        Some(Self {})
    }

    fn cons_list(_x: Self::Hd, _xs: Self::Tl) -> Self {
        Self {}
    }

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

    fn elem_type_vec(_t: PhantomData<Self>) -> Result<Vec<ElemType>, ElemsPopError> {
        Ok(vec![])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cons<T: Elems, U: IsList> {
    hd: T,
    tl: U,
}

pub struct IterCons<T: Elems, U: IsList> {
    hd: <T as IntoIterator>::IntoIter,
    tl: <U as IntoIterator>::IntoIter,
}

impl<T: Elems, U: IsList> IntoIterator for Cons<T, U> {
    type Item = Elem;
    type IntoIter = IterCons<T, U>;

    fn into_iter(self) -> Self::IntoIter {
        IterCons {
            hd: self.hd.into_iter(),
            tl: self.tl.into_iter(),
        }
    }
}

impl<T: Elems, U: IsList> Iterator for IterCons<T, U> {
    type Item = Elem;

    fn next(&mut self) -> Option<Self::Item> {
        self.hd.next().or_else(|| self.tl.next())
    }
}

impl<T: Elems, U: IsList> IsList for Cons<T, U> {
    type Hd = T;
    type Tl = U;

    fn empty_list() -> Option<Self> where Self: Sized {
        None
    }

    fn cons_list(x: Self::Hd, xs: Self::Tl) -> Self {
        Cons {
            hd: x,
            tl: xs,
        }
    }

    fn is_empty(&self) -> bool {
        false
    }

    fn hd(self) -> Self::Hd {
        self.hd
    }

    fn tl(self) -> Self::Tl {
        self.tl
    }

    fn elem_type_vec(_t: PhantomData<Self>) -> Result<Vec<ElemType>, ElemsPopError> {
        let elem_type_hd = Elems::elem_type(PhantomData::<T>)?;
        let mut elem_type_vec_tl = IsList::elem_type_vec(PhantomData::<U>)?;
        elem_type_vec_tl.insert(0, elem_type_hd);
        Ok(elem_type_vec_tl)
    }
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

    fn returning(&self) -> Option<Elem>;
    fn type_of(t: PhantomData<Self>) -> Result<Type, ElemsPopError>;
}

impl<T, U> IOList for Cons<T, U>
where
    T: IElems,
    U: IOList,
{
    type Return = U::Return;

    fn returning(&self) -> Option<Elem> {
        self.tl.returning()
    }

    // TODO: test
    fn type_of(_t: PhantomData<Self>) -> Result<Type, ElemsPopError> {
        let num_elem_type_hd = <<T as Elems>::N as Unsigned>::to_usize();
        let elem_type_hd = Elems::elem_type(PhantomData::<T>)?;
        let mut type_tl = IOList::type_of(PhantomData::<U>)?;

        type_tl.prepend_inputs(num_elem_type_hd, elem_type_hd);
        Ok(type_tl)
    }
}


#[derive(Clone, Debug)]
pub struct ConsOut<T, U>
where
    T: IOElems,
    U: IList,
{
    cons: Cons<T, U>,
}

impl<T: IOElems, U: IList> IntoIterator for ConsOut<T, U> {
    type Item = Elem;
    type IntoIter = IterCons<T, U>;

    fn into_iter(self) -> Self::IntoIter {
        self.cons.into_iter()
        // IterCons {
        //     cons: self.cons,
        //     at_head: true,
        // }
    }
}

impl<T, U> IsList for ConsOut<T, U>
where
    T: IOElems,
    U: IList
{
    type Hd = T;
    type Tl = U;

    fn empty_list() -> Option<Self> where Self: Sized {
        None
    }

    fn cons_list(x: Self::Hd, xs: Self::Tl) -> Self {
        ConsOut {
            cons: Cons {
                hd: x,
                tl: xs,
            },
        }
    }

    fn is_empty(&self) -> bool {
        self.cons.is_empty()
    }

    fn hd(self) -> Self::Hd {
        self.cons.hd()
    }

    fn tl(self) -> Self::Tl {
        self.cons.tl()
    }

    fn elem_type_vec(_t: PhantomData<Self>) -> Result<Vec<ElemType>, ElemsPopError> {
        IsList::elem_type_vec(PhantomData::<Cons<T, U>>)
    }
}

impl<T, U> IOList for ConsOut<T, U>
where
    T: IOElems,
    U: IList,
{
    type Return = T;

    fn returning(&self) -> Option<Elem> {
        self.cons.hd.returning()
    }

    // TODO: add info to errors
    fn type_of(_t: PhantomData<Self>) -> Result<Type, ElemsPopError> {
        // let num_elem_type_hd = <<T as Elems>::N as Unsigned>::to_usize();
        let mut type_hd = IOElems::type_of(PhantomData::<T>)?;
        let elem_type_tl = IsList::elem_type_vec(PhantomData::<U>)?;

        type_hd.append_inputs(elem_type_tl);
        Ok(type_hd)
    }
}





pub trait IsInstructionT: Clone + Debug + PartialEq {
    type IO: IOList;
    type Error: AnError;

    fn name(x: PhantomData<Self>) -> String;
    fn run(&self, x: &Self::IO) -> Result<(), Self::Error>;
}

#[derive(Clone, Debug, Error)]
pub enum InstructionError {
    #[error("InstructionError::ElemsPopError:\n{0}")]
    ElemsPopError(ElemsPopError),

    #[error("RawInstructionError:\n{0}")]
    RawInstructionError(String),

    #[error("MissingOutput:\n{instruction}\n\n{stack_input}")]
    // TODO: more granular error typing
    MissingOutput {
        instruction: String,
        stack_input: String,
    },

    #[error("InstructionError::RestackError:\n{0}")]
    RestackError(RestackError),

    #[error("InstructionError::DebugJsonError:\n{0}")]
    DebugJsonError(Arc<serde_json::Error>),
}

pub trait IsStackInstruction: Debug {
    fn name(&self) -> String;
    fn type_of(&self) -> Result<Type, ElemsPopError>;
    fn stack_run(&self, stack: &mut Stack) -> Result<(), InstructionError>;
}

impl<T> IsStackInstruction for T
where
    T: IsInstructionT,
{
    fn name(&self) -> String {
        IsInstructionT::name(PhantomData::<Self>)
    }

    fn type_of(&self) -> Result<Type, ElemsPopError> {
        IOList::type_of(PhantomData::<<T as IsInstructionT>::IO>)
    }

    fn stack_run(&self, stack: &mut Stack) -> Result<(), InstructionError> {
        let stack_input = &IsList::pop(PhantomData::<<T as IsInstructionT>::IO>, stack)
            .map_err(|e| InstructionError::ElemsPopError(e))?;
        self.run(stack_input)
            .map_err(|e| InstructionError::RawInstructionError(format!("{:?}", e)))?;
        let output_value = stack_input
            .returning()
            .ok_or_else(|| InstructionError::MissingOutput {
                instruction: format!("{:?}", self),
                stack_input: format!("{:?}", stack_input),
            })?;
        stack.push(output_value);
        Ok(())
    }
}



#[derive(Clone, Debug)]
pub struct Instrs {
    // TODO: replace Result with Either?
    pub instrs: Vec<Result<Arc<dyn IsStackInstruction>, Restack>>,
}

// fn example_instrs() -> Instrs {
//     Instrs {
//         instrs: vec![
//             Arc::new(Concat {}),
//             Arc::new(AssertTrue {}),
//             Arc::new(Push { push: () }),
//             Arc::new(HashSha256 {}),
//             Arc::new(Slice {}),
//             Arc::new(Index {}),
//             Arc::new(ToJson {}),
//             Arc::new(Lookup {}),
//             Arc::new(UnpackJson { t: PhantomData::<()> }),
//             Arc::new(StringToBytes {}),
//             Arc::new(CheckLe {}),
//             Arc::new(CheckLt {}),
//             Arc::new(CheckEq {})
//         ],
//     }
// }


impl Instrs {
    pub fn new() -> Self {
        Instrs {
            instrs: vec![],
        }
    }

    pub fn run(&self, stack: &mut Stack) -> Result<(), InstructionError> {
        for instr_or_restack in &self.instrs {
            stack.debug().map_err(|e| InstructionError::DebugJsonError(Arc::new(e)))?;
            println!("------------------------------------------------------------------------------------------");
            println!("#: {:?}\n", instr_or_restack);
            match instr_or_restack {
                Ok(instr) => {
                    let mut instr_type = instr.type_of();
                    stack.debug_type();
                    format!("");

                    match instr_type {
                        Ok(instr_type) => {
                            println!("instr: {}\n", instr_type);
                            let mut mut_instr_type = instr_type.clone();
                            match mut_instr_type
                                .specialize_to_input_stack(stack
                                                           .clone()
                                                           .stack
                                                           .into_iter()
                                                           .map(|x| x.elem_type(vec![]))
                                                           .collect()) {
                                Ok(specialized) => println!("specialized: {}\n", mut_instr_type),
                                Err(e) => println!("specialization failed: {}\n", e),
                            }
                        },
                        Err(e) => println!("instr type_of errror: {}\n", e),
                    }
                    println!("");
                    instr.stack_run(stack)?
                },
                Err(restack) => {
                    println!("restack: {:?}\n", restack);
                    restack.run(&mut stack.stack)
                        .map_err(|e| InstructionError::RestackError(e))?
                },
            }
        }
        Ok(())
    }

    pub fn instr(&mut self, instr: impl IsStackInstruction + 'static) -> () {
        self.instrs.push(Ok(Arc::new(instr)))
    }

    pub fn restack(&mut self, restack: Restack) -> () {
        self.instrs.push(Err(restack))
    }
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

    fn name(_x: PhantomData<Self>) -> String {
        "concat".to_string()
    }

    fn run(&self, x: &Self::IO) -> Result<(), Self::Error> {
        let y = x.clone().hd();
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
    // TODO: replace w/ Empty
    type Error = AssertTrueError;

    fn name(_x: PhantomData<Self>) -> String {
        "assert_true".to_string()
    }

    fn run(&self, x: &Self::IO) -> Result<(), Self::Error> {
        let array = x.clone().hd().singleton.array;
        let returning = x.clone().hd().returning;
        if array[0] {
            returning.returning(true);
            Ok(())
        } else {
            Err(AssertTrueError {})
        }
    }
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Push<T: AnElem> {
    pub push: T,
}

impl<T: AnElem> IsInstructionT for Push<T> {
    type IO = ConsOut<ReturnSingleton<T, U0>, Nil>;
    type Error = Empty;

    fn name(_x: PhantomData<Self>) -> String {
        format!("push_{:?}", AnElem::elem_symbol(PhantomData::<T>))
    }

    fn run(&self, x: &Self::IO) -> Result<(), Self::Error> {
        x.clone().hd().returning.returning(self.push.clone());
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

    fn name(_x: PhantomData<Self>) -> String {
        "sha256".to_string()
    }

    fn run(&self, x: &Self::IO) -> Result<(), Self::Error> {
        let array = x.clone().hd().singleton.array;
        let returning = x.clone().hd().returning;
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

    fn name(_x: PhantomData<Self>) -> String {
        "slice".to_string()
    }

    fn run(&self, x: &Self::IO) -> Result<(), Self::Error> {
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

    fn name(_x: PhantomData<Self>) -> String {
        "index".to_string()
    }

    fn run(&self, x: &Self::IO) -> Result<(), Self::Error> {
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

    fn name(_x: PhantomData<Self>) -> String {
        "to_json".to_string()
    }

    fn run(&self, x: &Self::IO) -> Result<(), Self::Error> {
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
    type IO = ConsOut<ReturnSingleton<Value, U0>,
                 Cons<Singleton<String, U1>,
                 Cons<Singleton<Map<String, Value>, U1>, Nil>>>;
    type Error = LookupError;

    fn name(_x: PhantomData<Self>) -> String {
        "lookup".to_string()
    }

    fn run(&self, x: &Self::IO) -> Result<(), Self::Error> {
        let returning = x.clone().hd().returning;
        let key = &x.clone().tl().hd().array[0];
        let map = &x.clone().tl().tl().hd().array[0];
        returning.returning(map.get(key)
           .ok_or_else(|| LookupError {
               key: key.clone(),
               map: map.clone(),
           })?.clone());
        Ok(())
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnpackJson<T: AnElem> {
    pub t: PhantomData<T>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnpackJsonError {}
impl AnError for UnpackJsonError {}

trait AJsonElem: AnElem {
    fn to_value(self) -> Value;
    fn from_value(t: PhantomData<Self>, x: Value) -> Option<Self>;
}

impl AJsonElem for () {
    fn to_value(self) -> Value {
        Value::Null
    }

    fn from_value(_t: PhantomData<Self>, x: Value) -> Option<Self> {
        match x {
            Value::Null => Some(()),
            _ => None,
        }
    }
}

impl AJsonElem for bool {
    fn to_value(self) -> Value {
        Value::Bool(self)
    }

    fn from_value(_t: PhantomData<Self>, x: Value) -> Option<Self> {
        match x {
            Value::Bool(y) => Some(y),
            _ => None,
        }
    }
}

impl AJsonElem for Number {
    fn to_value(self) -> Value {
        Value::Number(self)
    }

    fn from_value(_t: PhantomData<Self>, x: Value) -> Option<Self> {
        match x {
            Value::Number(y) => Some(y),
            _ => None,
        }
    }
}

impl AJsonElem for String {
    fn to_value(self) -> Value {
        Value::String(self)
    }

    fn from_value(_t: PhantomData<Self>, x: Value) -> Option<Self> {
        match x {
            Value::String(y) => Some(y),
            _ => None,
        }
    }
}

impl AJsonElem for Vec<Value> {
    fn to_value(self) -> Value {
        Value::Array(self)
    }

    fn from_value(_t: PhantomData<Self>, x: Value) -> Option<Self> {
        match x {
            Value::Array(y) => Some(y),
            _ => None,
        }
    }
}

impl AJsonElem for Map<String, Value> {
    fn to_value(self) -> Value {
        Value::Object(self)
    }

    fn from_value(_t: PhantomData<Self>, x: Value) -> Option<Self> {
        match x {
            Value::Object(y) => Some(y),
            _ => None,
        }
    }
}

impl<T: AJsonElem> IsInstructionT for UnpackJson<T> {
    type IO = ConsOut<ReturnSingleton<T, U0>,
                       Cons<Singleton<Value, U1>, Nil>>;
    type Error = UnpackJsonError;

    fn name(_x: PhantomData<Self>) -> String {
        "unpack_json".to_string()
    }

    fn run(&self, x: &Self::IO) -> Result<(), Self::Error> {
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

    fn name(_x: PhantomData<Self>) -> String {
        "string_to_bytes".to_string()
    }

    fn run(&self, x: &Self::IO) -> Result<(), Self::Error> {
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

    fn name(_x: PhantomData<Self>) -> String {
        "check_le".to_string()
    }

    fn run(&self, x: &Self::IO) -> Result<(), Self::Error> {
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

    fn name(_x: PhantomData<Self>) -> String {
        "check_lt".to_string()
    }

    fn run(&self, x: &Self::IO) -> Result<(), Self::Error> {
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

    fn name(_x: PhantomData<Self>) -> String {
        "check_eq".to_string()
    }

    fn run(&self, x: &Self::IO) -> Result<(), Self::Error> {
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
//     fn elem_symbol(_t: PhantomData<Self>) -> ElemType {
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






