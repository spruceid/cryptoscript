
pub(crate) mod map;

use std::fmt::{Display, Formatter};
use std::fmt;

/// A Type ID, represented as a usize
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypeId {
    type_id: usize,
}

impl Display for TypeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
       write!(f, "type#{}", self.type_id)
    }
}

impl TypeId {
    /// New TypeId with the given ID
    pub fn new(type_id: usize) -> Self {
        Self {
            type_id,
        }
    }

    /// format!("t{}", self.type_id)
    pub fn debug(&self) -> String {
        format!("t{}", self.type_id)
    }

    /// Subtract one or return None if 0
    pub fn previous(&self) -> Option<Self> {
        self.type_id.checked_sub(1).map(|type_id| Self {
            type_id,
        })
    }

    // TODO: test by checking:
    // xs.map(TypeId).fold(x, offset) = TypeId(xs.fold(x, +))
    /// Offset (add) one TypeId to another
    pub fn offset(&self, offset: TypeId) -> Self {
        TypeId {
            type_id: self.type_id + offset.type_id,
        }
    }

    /// Replaces "from" TypeId with "to" TypeId.
    ///
    /// For compatibility with update_type_id in Context, etc.
    pub fn update_type_id(&self, from: Self, to: Self) -> Self {
        if *self == from {
            to
        } else {
            *self
        }
    }
}

