use crate::an_elem::AnElem;

use std::sync::{Arc, Mutex};

/// AnElem that can be returned, e.g. using IOElems or IOList.
///
/// In other words, a "typed return slot": use Return::new() to initialize an empty slot
#[derive(Clone, Debug)]
pub struct Return<T: AnElem> {
    return_value: Arc<Mutex<Option<T>>>,
}

impl<T: AnElem> Return<T> {
    /// New Return slot with nothing in it
    pub fn new() -> Self {
        Return {
            return_value: Arc::new(Mutex::new(None)),
        }
    }

    // TODO: throw error if try_lock fails
    /// Return the given return_value, overwriting any existing value.
    ///
    /// Panics if Mutex::try_lock fails
    pub fn returning(&self, return_value: T) {
        let mut lock = (*self.return_value).try_lock();
        if let Ok(ref mut mutex) = lock {
            **mutex = Some(return_value)
        } else {
            panic!("returning: try_lock failed")
        }
    }

    // TODO: throw error if try_lock fails
    /// The stored return_value, or None if nothing has been returned yet.
    ///
    /// Panics if Mutex::try_lock fails
    pub fn returned(&self) -> Option<T> {
        let mut lock = (*self.return_value).try_lock();
        if let Ok(ref mut mutex) = lock {
            (**mutex).clone()
        } else {
            panic!("returning: try_lock failed")
        }
    }
}

