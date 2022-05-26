use crate::elem::Elem;
use crate::types::Type;
use crate::elems::{Elems, ElemsPopError};
use crate::elems_input::IElems;
use crate::elems_input_output::IOElems;
use crate::elems_list::IsList;
use crate::elems_list_cons::Cons;

use std::marker::PhantomData;

use typenum::marker_traits::Unsigned;

/// Input-output type of an instruction
pub trait IOList: IsList {
    /// Returned IOElems
    type Return: IOElems;

    // TODO: rename to returned a la Return.
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

