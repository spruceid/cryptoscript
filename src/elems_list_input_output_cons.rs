use crate::elem::Elem;
use crate::elem_type::StackType;
use crate::types::Type;
use crate::elems::ElemsPopError;
use crate::elems_input_output::IOElems;
use crate::elems_list::IsList;
use crate::elems_list_cons::{Cons, IterCons};
use crate::elems_list_input::IList;
use crate::elems_list_input_output::IOList;

use std::marker::PhantomData;
use std::fmt::Debug;

/// Cons whose hd type is restricted to IOElems and whose tl type is restricted to IList
///
/// This ensures that there can only be one ConsOut per IOList.
/// By restricting instances of IOList, it must contain exactly one ConsOut.
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

