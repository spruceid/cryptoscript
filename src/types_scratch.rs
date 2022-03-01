// use crate::elem::{Elem, ElemSymbol};
use crate::stack::{Stack, StackError};
use crate::types::{AnElem, AnError, Nil, Teq, TEq, TypeName};

use std::iter::{FromIterator};

use std::fmt;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::sync::Arc;

use generic_array::{GenericArray, ArrayLength};

// #[derive(Clone)]
// struct Args<T: DynTrait, N: ArrayLength<Arc<<T as DynTrait>::Dyn>>> {
//     args: GenericArray<Arc<<T as DynTrait>::Dyn>, N>,
// }

// #[derive(Clone)]
// struct Returning<R, T, N> {
//     r: PhantomData<R>,
//     args: Args<T, N>,
// }



// fn cons<U, V: AnElem + Trait<U, V>>(self, u: PhantomData<U>, x: V) -> ConsT<U, V, Self> where Self: Sized;
#[derive(Clone, PartialEq, Eq)]
pub struct Cons<T, U: IList> {
    hd: T,
    tl: U,
}


pub trait IList {}

impl IList for Nil {}
impl<T: AnElem, U: IList> IList for Cons<T, U> {}

pub trait IOList: IList {
    type Return;
}


// #[derive(Clone, PartialEq, Eq)]
pub struct ConsOut<R, T: AnElem, U: IList> {
    r: PhantomData<R>,
    hd: T,
    tl: U,
}

impl<R, T: AnElem, U: IList> IList for ConsOut<R, T, U> {}
impl<R, T: AnElem, U: IList> IOList for ConsOut<R, T, U> {
    type Return = R;
}
impl<T: AnElem, U: IOList> IOList for Cons<T, U> {
    type Return = <U as IOList>::Return;
}

pub trait IsInstructionT: std::fmt::Debug {
    type In: IOList;
    // type Out: AnElem;
    type Error: AnError;

    fn run(&self, x: Self::In) -> Result<<Self::In as IOList>::Return, Self::Error>;
}

// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
// struct Concat<T: AnElem> {
//     t: PhantomData<T>,
// }
// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
// struct ConcatError {}
// impl AnError for ConcatError {}

// impl<T: AnElem + IntoIterator + FromIterator<<T as IntoIterator>::Item>> IsInstruction for Concat<T> {
//     type In = (T, T);
//     type Out = T;
//     type Error = Empty;

//     fn run(&self, x: Self::In) -> Result<Self::Out, Self::Error> {
//         let (lhs, rhs) = x;
//         Ok(lhs.into_iter().chain(rhs.into_iter()).collect())
//     }
// }










//////////////////////////////////////////////////////////////////////////////////////////////
//#[derive(Clone, PartialEq, Eq)]
//pub struct ConsT<T, U: AnElem + Trait<T, U>, V: TList> {
//    t: PhantomData<T>,
//    hd: U,
//    tl: V,
//}

//// + IntoIterator<Item = Elem>
//pub trait TList: Clone {
//    type T;
//    type Hd: AnElem + Trait<Self::T, Self::Hd>;
//    type Tl: TList;

//    fn is_empty(&self) -> bool;
//    fn hd(&self) -> Self::Hd;
//    fn tl(&self) -> Self::Tl;
//    fn cons<U, V: AnElem + Trait<U, V>>(self, u: PhantomData<U>, x: V) -> ConsT<U, V, Self> where Self: Sized;
//    fn pop(x: PhantomData<Self>, stack: &mut Stack) -> Result<Self, StackError>;
//}

//impl TList for Nil {
//    type T = Nil;
//    type Hd = ();
//    type Tl = Nil;

//    fn is_empty(&self) -> bool {
//        true
//    }

//    fn hd(&self) -> Self::Hd {
//        ()
//    }

//    fn tl(&self) -> Self::Tl {
//        Self {}
//    }

//    fn cons<U, V>(self, u: PhantomData<U>, x: V) -> ConsT<U, V, Self>
//    where
//        V: AnElem + Trait<U, V>,
//        Self: Sized,
//    {
//        ConsT {
//            t: PhantomData,
//            hd: x,
//            tl: self,
//        }
//    }

//    fn pop(_x: PhantomData<Self>, _stack: &mut Stack) -> Result<Self, StackError> {
//        Ok(Nil {})
//    }
//}

//impl<T, U, V> TList for ConsT<T, U, V>
//where
//    T: Clone,
//    U: AnElem + Trait<T, U>,
//    V: TList,
//{
//    type T = T;
//    type Hd = U;
//    type Tl = V;

//    fn is_empty(&self) -> bool {
//        false
//    }

//    fn hd(&self) -> Self::Hd {
//        self.hd.clone()
//    }

//    fn tl(&self) -> Self::Tl {
//        self.tl.clone()
//    }

//    fn cons<A, B>(self, u: PhantomData<A>, x: B) -> ConsT<A, B, Self>
//    where
//        B: AnElem + Trait<A, B>,
//        Self: Sized,
//    {
//        ConsT {
//            t: PhantomData,
//            hd: x,
//            tl: self,
//        }
//    }

//    // TODO: add better errors
//    fn pop(_x: PhantomData<Self>, stack: &mut Stack) -> Result<Self, StackError> {
//        let hd_elem = stack.pop()?;

//        // TODO: IMPLEMENT
//        Err(StackError::EmptyStack)

//        // Ok(Cons {
//        //     hd: AnElem::is_elem(PhantomData::<Self::Hd>).from_elem(hd_elem.clone()).ok_or_else(|| StackError::UnexpectedElemType {
//        //         expected: AnElem::is_elem(PhantomData::<Self::Hd>).elem_symbol(),
//        //         found: hd_elem.clone(),
//        //         stack: stack.clone(),
//        //     })?,
//        //     tl: Self::Tl::pop(PhantomData, stack)?,
//        // })
//    }
//}







// impl<T> DecTrait<Nil> for T {
//     fn dec(_t: PhantomData<Nil>, _s: PhantomData<Self>) -> Result<Arc<dyn Trait<Nil, Self>>, DecTraitError> {
//         Ok(Arc::new(()))
//     }
// }

// // : DecTrait<T>
// // : core::fmt::Debug
// pub trait Trait<T, U: AnElem> {}

// impl<T, U> Trait<Nil, T> for U
// where
//     T: AnElem,
// {}

// impl<T, U, V, W> Trait<Cons<T, U>, V> for W
// where
//     T: AnElem,
//     U: HList,
//     V: AnElem,
//     W: Trait<T, V>,
//     W: Trait<U, V>,
// {}

// #[derive(Debug)]
// pub struct IterTrait {}
// impl<T, U> Trait<IterTrait, T> for U
// where
//     T: AnElem + IntoIterator + FromIterator<<T as IntoIterator>::Item>,
// {}


// fn ok<T>(x: T) -> T
// where
//     // T: AnElem + IntoIterator + FromIterator<<T as IntoIterator>::Item>,
//     T: AnElem,
//     (): Trait<IterTrait, T>,
// {
//     // x
//     x.into_iter().collect()
// }

// TODO: remove Trait<T> entirely, it simply doesn't work!

// #[derive(Clone)]
// pub struct DynTrait<T, U: AnElem> {
//     dyn_trait: Arc<dyn Trait<T, U>>,
// }

// impl<T: AnElem + TypeName> std::fmt::Debug for dyn Trait<IterTrait, T> {
//     fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
//         write!(f, "IterTrait {}", TypeName::type_name(PhantomData::<T>))
//     }
// }

// impl<T: AnElem> AnElem for DynTrait<IterTrait, T> {
//     fn is_elem(x: PhantomData<Self>) -> IsElem<Self> where Self: Sized {
//         AnElem::is_elem(Ph

// impl<T: AnElem> IntoIterator for dyn Trait<IterTrait, T> {}
// impl<T: AnElem> FromIterator<<T as IntoIterator>::Item> for dyn Trait<IterTrait, T> {}

// #[derive(Debug)]
// pub struct TeqTrait<T> {
//     t: PhantomData<T>
// }
// impl<T, U, V> Trait<TeqTrait<T>, U> for V
// where
//     U: AnElem + Teq<T>,
// {}


//////////////////////////////////////////////////////////////////////////////////////////////
//pub trait ElemTrait<T: AnElem> {
//    // // sym
//    // fn transport_trait<T: AnElem, U: AnElem>(eq: TEq<T, U>, xs: Arc<dyn Trait<Self, U>>) -> Arc<dyn Trait<Self, T>>;
//    fn dec(s: PhantomData<Self>, t: PhantomData<T>) -> Result<Arc<dyn Trait<Self, T>>, TraitError>;
//}

//impl<T: AnElem> ElemTrait<T> for Nil {
//    fn dec(s: PhantomData<Self>, t: PhantomData<T>) -> Result<Arc<dyn Trait<Self, T>>, TraitError> {
//        Ok(Arc::new(()))
//    }
//}

//impl ElemTrait<()> for IterTrait {
//    fn dec(s: PhantomData<Self>, t: PhantomData<()>) -> Result<Arc<dyn Trait<Self, ()>>, TraitError> {
//        Err(TraitError::TODO)
//    }
//}

//impl ElemTrait<Vec<u8>> for IterTrait {
//    fn dec(s: PhantomData<Self>, t: PhantomData<Vec<u8>>) -> Result<Arc<dyn Trait<Self, Vec<u8>>>, TraitError> {
//        Ok(Arc::new(()))
//    }
//}

// impl<T> dyn Trait<IterTrait, T> {
//     fn ok(&self, x: T) -> T {
//         self.into_iter().collect()
//     }
// }

//     fn transport_trait<T: AnElem, U: AnElem>(eq: TEq<T, U>, xs: Arc<dyn Trait<Self, U>>) -> Arc<dyn Trait<Self, T>> {
//         Arc::new(())
//     }

//     fn dec<T: AnElem>(s: PhantomData<Self>, t: PhantomData<T>) -> Result<Arc<dyn Trait<Self, T>>, TraitError> {
//         match AnElem::is_elem(PhantomData<T>) {
//             Unit(eq) => self.transport_trait(eq, Arc::new(())),
//             Bool(eq) => self.transport_trait(eq, Arc::new(())),
//             Number(eq) => self.transport_trait(eq, Arc::new(())),
//             Bytes(eq) => self.transport_trait(eq, Arc::new(())),
//             String(eq) => self.transport_trait(eq, Arc::new(())),
//             Array(eq) => self.transport_trait(eq, Arc::new(())),
//             Object(eq) => self.transport_trait(eq, Arc::new(())),
//             Json(eq) => self.transport_trait(eq, Arc::new(())),

//     }
// }


// impl ElemTrait for IterTrait {
//     fn transport_trait<T: AnElem, U: AnElem>(eq: TEq<T, U>, xs: Arc<dyn Trait<Self, U>>) -> Arc<dyn Trait<Self, T>> {
//         Arc::new(())
//     }

//     fn dec<T: AnElem>(s: PhantomData<Self>, t: PhantomData<T>) -> Result<Arc<dyn Trait<Self, T>>, TraitError> {
//         match AnElem::is_elem(PhantomData<T>) {
//             Unit(eq) => self.transport_trait(eq, Arc::new(())),
//             Bool(eq) => self.transport_trait(eq, Arc::new(())),
//             Number(eq) => self.transport_trait(eq, Arc::new(())),
//             Bytes(eq) => self.transport_trait(eq, Arc::new(())),
//             String(eq) => self.transport_trait(eq, Arc::new(())),
//             Array(eq) => self.transport_trait(eq, Arc::new(())),
//             Object(eq) => self.transport_trait(eq, Arc::new(())),
//             Json(eq) => self.transport_trait(eq, Arc::new(())),

//     }
// }

// pub enum TraitError {
//     TODO,
// }

// pub trait AnElem: Clone + std::fmt::Debug {
//     fn is_elem(x: PhantomData<Self>) -> IsElem<Self> where Self: Sized;
// }



// pub trait DynTrait: ElemTrait {
//     type Dyn;

//     fn from_trait<T>(t: PhantomData<T>, x: Arc<dyn Trait<Self, T>>) -> Self::Dyn;
// }

// impl DynTrait for Nil {
//     type Dyn = Arc<dyn std::fmt::Debug>;

//     fn from_trait<T>(t: PhantomData<T>, x: Arc<dyn Trait<Self, T>>) -> Self::Dyn {
//         Arc::new(())
//     }
// }

// #[derive(Debug)]
// pub struct IterTrait {}

// impl DynTrait for IterTrait 

// impl<T, U> Trait<IterTrait, T> for U
// where
//     T: AnElem + IntoIterator + FromIterator<<T as IntoIterator>::Item>,
// {}


// pub trait TraitArray<T: ElemTrait, N> {
//     fn run_trait_array<U>(&self) -> () where (): Trait<T, U>

// pub trait TraitArray<T: ElemTrait, N> {
//     fn run_trait_array<U>(&self) -> () where (): Trait<T, U>

// pub struct Foo<T: ElemTrait, N> {
//     GenericArray<U, N>

// pub trait TraitArray<T: ElemTrait, N: ArrayLength<Self>> {
//     // fn ok(&self) -> 

//     // fn ok(&self) -> ();
// }

// pub trait TraitArray<T: ElemTrait> {
//     type ElemT;
//     type N: ArrayLength<Self::ElemT>;

//     fn has_trait(&self) -> Arc<dyn Trait<T, Self::ElemT>>;
//     fn array(&self) -> GenericArray<Self::ElemT, Self::N>;
// }
