use crate::elem::Elem;
use crate::elem_type::StackType;
use crate::elems::{Elems, ElemsPopError};
use crate::elems_list::IsList;

use std::marker::PhantomData;
use std::fmt::{self, Debug, Formatter};

use typenum::marker_traits::Unsigned;

/// A non-empty list of Elems, where the first element is explicitly provided
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cons<T: Elems, U: IsList> {
    /// The head of the list, which must be Elems
    pub hd: T,

    /// The tail of the list, which IsList
    pub tl: U,
}

/// IntoIterator applied to Cons::hd and Cons::tl
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

    // fn is_empty(&self) -> bool {
    //     false
    // }

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

