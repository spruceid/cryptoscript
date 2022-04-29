use std::marker::PhantomData;

use thiserror::Error;

/// An empty enum or impossible-to-inhabit type
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum Empty {}

impl Empty {
    /// Given Empty, produce anything
    pub fn absurd<T>(&self, _p: PhantomData<T>) -> T {
        match *self {}
    }
}

