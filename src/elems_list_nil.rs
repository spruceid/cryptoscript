use crate::elem::Elem;
use crate::elem_type::StackType;
use crate::elems_list::IsList;
use crate::elems_singleton::Singleton;
use crate::elems::ElemsPopError;

use std::marker::PhantomData;
use std::fmt::Debug;

use generic_array::sequence::GenericSequence;
use generic_array::typenum::U0;
use generic_array::GenericArray;

/// An empty IsList, i.e. an empty list of Elems
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Nil {}

impl Iterator for Nil {
    type Item = Elem;

    fn next(&mut self) -> Option<Self::Item> {
        None
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

    // fn is_empty(&self) -> bool {
    //     true
    // }

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

