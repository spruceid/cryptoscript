use crate::elem::Elem;
use crate::elem_type::{ElemType, StackType};
use crate::an_elem::AnElem;
use crate::stack::Stack;
use crate::types::{Context, Type, Nil};
use crate::elems_singleton::Singleton;
use crate::elems_or::Or;
use crate::elems::{Elems, ElemsPopError};

use std::marker::PhantomData;
use std::fmt::{self, Debug, Formatter};
use std::sync::{Arc, Mutex};

use generic_array::sequence::GenericSequence;
use generic_array::typenum::U0;
use generic_array::{GenericArray, ArrayLength};
use typenum::marker_traits::Unsigned;

// TODO: split out
// - IOElems
//     + IElems
//     + Return
// - IOElems_singleton
// - IOElems_or
// - IsList
// - IOList
//     + IOList_cons_out


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



impl<T, N> IElems for Singleton<T, N>
where
    T: AnElem,
    N: ArrayLength<T> + Debug,
{}

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
    pub singleton: Singleton<T, N>,
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
        let mut type_tl = IOElems::type_of(PhantomData::<U>)
            .map_err(|e| ElemsPopError::ReturnOrTl(Arc::new(e)))?;
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

    fn stack_type(t: PhantomData<Self>) -> Result<StackType, ElemsPopError>;

    fn pop(_x: PhantomData<Self>, stack: &mut Stack) -> Result<Self, ElemsPopError>
    where
        Self: Sized,
    {
        match <Self as IsList>::empty_list() {
            Some(x) => Ok(x),
            None => {
                let original_stack = stack.clone();
                let x = <Self::Hd as Elems>::pop(PhantomData, stack).or_else(|e| Err(ElemsPopError::IsListHd {
                    stack_type: IsList::stack_type(PhantomData::<Self>)?,
                    elem_set: Elems::elem_type(PhantomData::<Self::Hd>)?,
                    stack_type_of: original_stack.clone().type_of(),
                    error: Arc::new(e),
                }))?;
                let xs = <Self::Tl as IsList>::pop(PhantomData, stack).or_else(|e| Err(ElemsPopError::IsListTl {
                    stack_type: IsList::stack_type(PhantomData::<Self>)?,
                    stack_type_of: original_stack.clone().type_of(),
                    error: Arc::new(e),
                }))?;
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

    fn stack_type(_t: PhantomData<Self>) -> Result<StackType, ElemsPopError> {
        Ok(StackType {
            types: vec![],
        })
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

impl<T, U> Debug for IterCons<T, U>
where
    T: Elems,
    U: IsList,
    <T as IntoIterator>::IntoIter: Debug,
    <U as IntoIterator>::IntoIter: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "Cons {{\n  hd: {:?},\n  tl: {:?}\n}}", self.hd, self.tl)
    }
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

    fn stack_type(_t: PhantomData<Self>) -> Result<StackType, ElemsPopError> {
        let elem_type_hd = Elems::elem_type(PhantomData::<T>)?;
        let elem_type_hd_count = <<T as Elems>::N as Unsigned>::to_usize();
        let mut stack_type_tl = IsList::stack_type(PhantomData::<U>)?;
        stack_type_tl.push_n(elem_type_hd, elem_type_hd_count);
        Ok(stack_type_tl)
    }
}





/// IsList and defines inputs, but no outputs. See IOList for more info
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

/// Input-output type of an instruction
pub trait IOList: IsList {
    /// Returned IOElems
    type Return: IOElems;

    /// Returned value, if set
    fn returning(&self) -> Option<Elem>;

    /// IOList's define a complete input/output Type, with exacly one return value
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

    fn stack_type(_t: PhantomData<Self>) -> Result<StackType, ElemsPopError> {
        IsList::stack_type(PhantomData::<Cons<T, U>>)
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
        let elem_type_tl = IsList::stack_type(PhantomData::<U>)?;
        type_hd.append_inputs(elem_type_tl);
        Ok(type_hd)
    }
}

