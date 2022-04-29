use crate::elems_input::IElems;
use crate::elems_list::IsList;
use crate::elems_list_nil::Nil;
use crate::elems_list_cons::Cons;

/// IsList that defines inputs, but no outputs. See IOList for more info
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

