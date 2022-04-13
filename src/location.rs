use serde::{Deserialize, Serialize};

/// Line number, 0-indexed
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LineNo {
    /// Line number
    pub line_no: usize,
}

impl From<usize> for LineNo {
    fn from(line_no: usize) -> Self {
        LineNo {
            line_no: line_no,
        }
    }
}

/// Index of an argument, e.g. Concat has two arguments with indices
/// [0, 1], in that order
pub type ArgumentIndex = usize;

/// Location of an input or output
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Location {
    line_no: LineNo,
    argument_index: ArgumentIndex,
    is_input: bool,
}

impl LineNo {
    /// From::from(2).in_at(3) is the position:
    /// - line_no: 2
    /// - argument_index: 3
    /// - is_input: true
    pub fn in_at(&self, argument_index: usize) -> Location {
        Location {
            line_no: *self,
            argument_index: argument_index,
            is_input: true,
        }
    }

    /// From::from(2).out_at(3) is the position:
    /// - line_no: 2
    /// - argument_index: 3
    /// - is_input: false
    pub fn out_at(&self, argument_index: usize) -> Location {
        Location {
            line_no: *self,
            argument_index: argument_index,
            is_input: false,
        }
    }
}
