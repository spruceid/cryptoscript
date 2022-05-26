use crate::elem::Elem;
use crate::elem_type::StackType;
use crate::stack::Stack;
use crate::elems::{Elems, ElemsPopError};

use std::marker::PhantomData;
use std::fmt::Debug;
use std::sync::Arc;

/// A non-empty ordered list of Elems
pub trait IsList: Clone + Debug + IntoIterator<Item = Elem> {
    /// The Hd Elems, or unit if empty
    type Hd: Elems;

    /// The rest of the list, or Nil
    type Tl: IsList;

    /// Return Self if empty
    fn empty_list() -> Option<Self> where Self: Sized;

    /// Cons Self::Hd with Self::Tl
    fn cons_list(x: Self::Hd, xs: Self::Tl) -> Self;

    /// Is it empty?
    fn is_empty(&self) -> bool {
        <Self as IsList>::empty_list().is_some()
    }

    /// Self::Hd can always be returned
    fn hd(self) -> Self::Hd;

    /// Self::Tl can always be returned
    fn tl(self) -> Self::Tl;

    // fn cons<T: Elems>(self, x: T) -> Cons<T, Self>
    // where
    //     Self: Sized,
    // {
    //     Cons {
    //         hd: x,
    //         tl: self,
    //     }
    // }

    /// The StackType of this list of Elems
    fn stack_type(t: PhantomData<Self>) -> Result<StackType, ElemsPopError>;

    /// Pop this type from an untyped Stack
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

