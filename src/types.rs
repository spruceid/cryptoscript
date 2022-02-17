use crate::restack::Restack;
use crate::elem::{Elem, ElemSymbol};

use std::ops::Range;

use enumset::{EnumSet, EnumSetType};
use serde::{Deserialize, Serialize};
use thiserror::Error;


// NEXT:
// - define a context of type variables (Vec<EnumSet<ElemSymbol>> === Map<uname: VarId, EnumSet<ElemSymbol>>)
// - define input/output stacks of type variables (Vec<uname: VarId>)
// - define unification/inference/typing rules/patterns

// Typing Overview:
// - calculate the number of in/out stack elements per instruction
//     + most consume 0..2 and produce one input
//     + exceptions are restack and assert_true
// - trace the stack type variables through the execution
//     + [ instruction ] -> [ (instruction, [stack_variable]) ], num_stack_variables
//     + map from type_var -> [ (instruction_location, (instruction), stack_location) ]
//         * instruction may/may-not be needed here
//         * stack_location differentiates between e.g. index number and iterable
//     + convert to a list of constraints
//     + resolve the list of constraints to a single type

// typing:
// - inference
// - checking against inferred or other type (this + inference = bidirecitonal)
// - unification
// - two categories of tests:
//   + property tests for typing methods themselves
//   + test that a function having a particular type -> it runs w/o type errors on such inputs

// Push(Elem),             // (t: type, elem: type(t)) : [] -> [ t ]
// Restack(Restack),       // (r: restack) : [ .. ] -> [ .. ]
// HashSha256,             // : [ bytes ] -> [ bytes ]
// CheckLe,                // : [ x, x ] -> [ bool ]
// CheckLt,                // : [ x, x ] -> [ bool ]
// CheckEq,                // : [ x, x ] -> [ bool ]
// Concat,                 // (t: type, prf: is_concat(t)) : [ t, t ] -> [ t ]
// Slice,                  // (t: type, prf: is_slice(t)) : [ int, int, t ] -> [ t ]
// Index,                  // (t: type, prf: is_index(t)) : [ int, t ] -> [ json ]
// Lookup,                 // [ string, object ] -> [ json ]
// AssertTrue,             // [ bool ] -> []
// ToJson,                 // (t: type) : [ t ] -> [ json ]
// UnpackJson(ElemSymbol), // (t: type) : [ json ] -> [ t ]
// StringToBytes,          // [ string ] -> [ bytes ]


#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub enum Instruction {
    Push(Elem),
    Restack(Restack),
    HashSha256,
    CheckLe,
    CheckLt,
    CheckEq,
    Concat,
    Slice,
    Index,
    Lookup,
    AssertTrue,
    ToJson,
    UnpackJson(ElemSymbol),
    StringToBytes,
}

impl Instruction {
    // (consumed_input_stack_size, produced_output_stack_size)
    pub fn stack_io_counts(&self) -> (usize, usize) {
        match self {
            Instruction::Push(_) => (0, 1),
            Instruction::Restack(restack) => restack.stack_io_counts(),
            Instruction::HashSha256 => (1, 1),
            Instruction::CheckLe => (2, 1),
            Instruction::CheckLt => (2, 1),
            Instruction::CheckEq => (2, 1),
            Instruction::Concat => (2, 1),
            Instruction::Slice => (3, 1),
            Instruction::Index => (2, 1),
            Instruction::Lookup => (2, 1),
            Instruction::AssertTrue => (1, 0),
            Instruction::ToJson => (1, 1),
            Instruction::UnpackJson(_) => (1, 1),
            Instruction::StringToBytes => (1, 1),
        }
    }

    // pub fn ty_sets(&self) -> (Vec<Ty>, Vec<Ty>) {
    //     match self {
    //         Instruction::Push(elem) => elem.push_ty_sets(),
    //         // Restack(restack) => restack.ty_sets(),
    //         Instruction::HashSha256 => (vec![ElemSymbol::Bytes.ty()], vec![ElemSymbol::Bytes.ty()]),
    //         // CheckLe,
    //         // CheckLt,
    //         Instruction::CheckEq => (vec![Ty::any(), Ty::any()], vec![ElemSymbol::Bool.ty()]),
    //         // Concat,
    //         // Slice,
    //         // Index,
    //         // Lookup,
    //         // AssertTrue,
    //         // ToJson,
    //         // UnpackJson(ElemSymbol),
    //         // StringToBytes,
    //         _ => panic!("infer_instruction: unimplemented"),
    //     }
    // }
}

pub type Instructions = Vec<Instruction>;

// pub type Stack = Vec<Elem>;



#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SrcRange {
    range: Range<usize>,
}

impl SrcRange {
    pub fn singleton(src_location: usize) -> Self {
        SrcRange {
            range: (src_location..src_location + 1),
        }
    }

    pub fn append(&self, other: Self) -> Result<Self, SrcRangeError> {
        if self.range.end + 1 == other.range.start {
            Ok(SrcRange { range: self.range.start..other.range.end })
        } else {
            Err(SrcRangeError::MismatchedRanges {
                lhs: self.clone(),
                rhs: other,
            })
        }
    }
}

#[derive(Debug, PartialEq, Error)]
pub enum SrcRangeError {
    #[error("SrcRange::append applied to non-contiguous ranges: lhs: {lhs:?}; rhs: {rhs:?}")]
    MismatchedRanges {
        lhs: SrcRange,
        rhs: SrcRange,
    },
}




#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Ty {
    ty_set: EnumSet<ElemSymbol>,
}

// #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
// pub struct TyUnifyLocation {
//     lhs: SrcRange,
//     rhs: SrcRange,
//     stack_position: usize,
// }

// impl Ty {
//     pub fn any() -> Self {
//         Ty {
//             ty_set: EnumSet::all(),
//         }
//     }

//     pub fn unify(&self, other: Self, location: TyUnifyLocation) -> Result<Self, TypeError> {
//         let both = self.ty_set.intersection(other.ty_set);
//         if both.is_empty() {
//             Err(TypeError::TyUnifyEmpty {
//                 lhs: self.clone(),
//                 rhs: other,
//                 location: location,
//             })
//         } else {
//             Ok(Ty {
//                 ty_set: both
//             })
//         }
//     }
// }




// // TODO: use in_ty/out_ty

// #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
// pub struct StackTy {
//     src_range: SrcRange,
//     in_ty: Vec<Ty>,
//     out_ty: Vec<Ty>,
// }

// impl StackTy {
//     // pub fn zip_extend_with<T>(xs: Vec<T>, ys: Vec<T>, fl: Fn(T) -> T, f: Fn(T, T)

//     pub fn unify_ty_sets(xs: Vec<Ty>, ys: Vec<Ty>, xs_src_range: SrcRange, ys_src_range: SrcRange) -> Result<Vec<Ty>, TypeError> {
//         let zip_len = cmp::max(xs.len(), ys.len());
//         let xs_extended = xs.iter().map(|z|Some(z)).chain(std::iter::repeat(None).take(zip_len - xs.len()));
//         let ys_extended = ys.iter().map(|z|Some(z)).chain(std::iter::repeat(None).take(zip_len - ys.len()));

//         xs_extended.zip(ys_extended).enumerate().map(|ixy| {
//             match ixy.1 {
//                 (None, None) => Err(TypeError::StackTyUnifyNone {
//                     lhs: xs.clone(),
//                     rhs: ys.clone(),
//                 }),
//                 (Some(x), None) => Ok(*x),
//                 (None, Some(y)) => Ok(*y),
//                 (Some(x), Some(y)) => Ok(x.unify(*y, TyUnifyLocation {
//                     lhs: xs_src_range.clone(),
//                     rhs: ys_src_range.clone(),
//                     stack_position: ixy.0,
//                 })?),
//             }

//         }).collect()
//     }

//     pub fn diff_ty_sets(xs: Vec<Ty>, ys: Vec<Ty>) -> Result<Vec<Ty>, TypeError> {
//         xs.iter().zip(ys.iter()).map(|x, y| {
//             x.difference(y)
//         }.collect()
//     }

//     pub fn union_ty_sets(xs: Vec<Ty>, ys: Vec<Ty>) -> Result<Vec<Ty>, TypeError> {
//         // pad lengths and zip (pad with empty)
//         xs.iter().zip(ys.iter()).map(|x, y| {
//             x.union(y)
//         }.collect()
//     }

//     pub fn unify(&self, other: Self) -> Result<Self, TypeError> {
//         let middle_ty = Self::unify_ty_sets(self.out_ty, other.in_ty, self.src_range, other.src_range);
//         let self_remainder = Self::diff_ty_sets(middle_ty, self.out_ty);
//         let other_remainder = Self::diff_ty_sets(middle_ty, other.in_ty);
//             in_ty: self.in_ty + self_remainder
//             out_ty: other.out_ty + other_remainder

//         StackTy {
//             src_range: self.src_range.append(other.src_range)?,
//             in_ty: Self::union_ty_sets(self.in_ty, self_remainder),
//             out_ty: Self::union_ty_sets(other.out_ty, other_remainder),
//         }
//     }

//     pub fn infer_instruction(instruction: &Instruction, src_location: usize) -> Self {
//         let instruction_ty_sets = instruction.ty_sets();
//         StackTy {
//             src_range: SrcRange::singleton(src_location),
//             in_ty: instruction_ty_sets.0,
//             out_ty: instruction_ty_sets.1,
//         }
//     }

//     // pub fn infer(instructions: Instructions) -> Result<Self, TypeError> {
//     //     instructions.iter().enumerate()
//     //         .map(|ix| Self::infer_instruction(ix.1, ix.0))
//     //         .reduce(|memo, x| memo.unify(x))

//     // }

// }



// #[derive(Debug, PartialEq, Error)]
// pub enum TypeError {
//     #[error("Ty::unify applied to non-intersecting types: lhs: {lhs:?}; rhs: {rhs:?}")]
//     TyUnifyEmpty {
//         lhs: Ty,
//         rhs: Ty,
//         location: TyUnifyLocation,
//     },

//     // should be impossible
//     #[error("StackTy::unify produced an attempt to unify None and None: lhs: {lhs:?}; rhs: {rhs:?}")]
//     StackTyUnifyNone {
//         lhs: Vec<Ty>,
//         rhs: Vec<Ty>,
//     },

//     #[error("attempt to unify types of non-contiguous locations: lhs: {0:?}")]
//     SrcRangeError(SrcRangeError),
// }

// impl From<SrcRangeError> for TypeError {
//     fn from(error: SrcRangeError) -> Self {
//         Self::SrcRangeError(error)
//     }
// }

