use crate::restack::{Restack, RestackError};
use crate::elem::{Elem, ElemSymbol};

use std::collections::BTreeMap;

use enumset::{EnumSet, enum_set};
use serde::{Deserialize, Serialize};
// use thiserror::Error;


// TODO: relocate
pub fn zip_then<A, B, FAB, FA, FB, R>(a: A, b: B, mut fab: FAB, mut fa: FA, mut fb: FB) -> Result<(), R>
where
    A: IntoIterator,
    B: IntoIterator,
    FAB: FnMut(<A as std::iter::IntoIterator>::Item, <B as std::iter::IntoIterator>::Item) -> Result<(), R>,
    FA: FnMut(<A as std::iter::IntoIterator>::Item) -> Result<(), R>,
    FB: FnMut(<B as std::iter::IntoIterator>::Item) -> Result<(), R>,
{
    let mut b_iter = b.into_iter();
    for x in a.into_iter() {
        match b_iter.next() {
            Some(y) => fab(x, y)?,
            None => fa(x)?,
        }
    }
    for y in b_iter {
        fb(y)?
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zip_then_longer_rhs() {
        let xs = vec![false, false, false, false, true];
        let ys = vec![true, true, true, true, false, true, true, false];
        let mut xs_out = vec![];
        let mut ys_out = vec![];
        let mut xs_out_remainder = vec![];
        let mut ys_out_remainder = vec![];
        assert_eq!(Ok::<(), ()>(()),
            zip_then(&xs,
                     &ys,
                     |x, y| {
                         xs_out.push(x);
                         ys_out.push(y);
                         Ok(())
                     },
                     |x| {
                         xs_out_remainder.push(x);
                         Ok(())
                        },
                     |y| {
                         ys_out_remainder.push(y);
                         Ok(())
                    }));
        xs_out.append(&mut xs_out_remainder);
        ys_out.append(&mut ys_out_remainder);
        assert_eq!(xs.iter().map(|x| *x).collect::<Vec<bool>>(), xs_out.iter().map(|&x| *x).collect::<Vec<bool>>());
        assert_eq!(ys.iter().map(|x| *x).collect::<Vec<bool>>(), ys_out.iter().map(|&x| *x).collect::<Vec<bool>>());
    }
}



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

pub type Instructions = Vec<Instruction>;

// pub type Stack = Vec<Elem>;






#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ElemType {
    type_set: EnumSet<ElemSymbol>,
}

impl ElemSymbol {
    pub fn elem_type(&self) -> ElemType {
        ElemType {
            type_set: EnumSet::only(*self),
        }
    }
}

impl Elem {
    pub fn elem_type(&self) -> ElemType {
        self.symbol().elem_type()
    }
}

impl ElemType {
    pub fn any() -> Self {
        ElemType {
            type_set: EnumSet::all(),
        }
    }

    pub fn concat_type() -> Self {
        ElemType {
            type_set:
                enum_set!(ElemSymbol::Bytes |
                          ElemSymbol::String |
                          ElemSymbol::Array |
                          ElemSymbol::Object),
        }
    }

    pub fn index_type() -> Self {
        ElemType {
            type_set:
                enum_set!(ElemSymbol::Array |
                          ElemSymbol::Object),
        }
    }

    pub fn slice_type() -> Self {
        Self::concat_type()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypeId {
    type_id: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Context {
    context: BTreeMap<TypeId, ElemType>,
    next_type_id: TypeId,
}

impl Context {
    pub fn new() -> Self {
        Context {
            context: BTreeMap::new(),
            next_type_id: TypeId {
                type_id: 0,
            },
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.context.keys().any(|x| *x >= self.next_type_id)
    }

    pub fn size(&self) -> usize {
        self.context.len()
    }

    pub fn push(&mut self, elem_type: ElemType) -> TypeId {
        let push_id = self.next_type_id;
        self.context.insert(push_id, elem_type);
        self.next_type_id = TypeId {
            type_id: push_id.type_id + 1,
        };
        push_id
    }

    // pub fn push_unified(&mut self, xs: Self, ys: Self, xi: TypeId, yi: TypeId) -> Self {
    //     _
    // }
}



#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Type {
    context: Context,
    i_type: Vec<TypeId>,
    o_type: Vec<TypeId>,
}

impl Type {
    // check whether all the TypeId's are valid
    pub fn is_valid(&self) -> bool {
        let next_type_id = self.context.next_type_id;
        self.context.is_valid() &&
        !(self.i_type.iter().any(|x| *x >= next_type_id) ||
          self.o_type.iter().any(|x| *x >= next_type_id))
    }

    // TODO:
    //     - use remapping of variables (always larger than max of both)
    //     - make method to simplify context
    //     - pretty-print Type

    // TODO: this is next, figure out how to represent/implement replacements of
    //     variables, e.g. starting larger than both or collecting maps (old_lhs -> new_lhs)

    // pub fn compose(&self, other: Self) -> Result<Self, TypeError> {
    //     let mut context = Context::new();

    //     let unified_overlap = self.o_type.iter().zip(other.i_type.iter()).map(|x, y| {
    //         (x, y, context.push_unified(self, other, x, y));
    //     });

    //     let remainder_overlap_iter = if self.o_type.len() <= other.i_type.len()) {
    //         other.i_type.iter().skip(self.o_type.len())
    //     } else {
    //         self.o_type.iter().skip(other.i_type.len())
    //     };

    //     let remainder_overlap = remainder_overlap_iter.map(|x| {
    //         (x, context.push(x));
    //     });

    //     1. iterate through (zip(self.o_type, other.i_type)) and unify the pairs into a new context
    //     2. collect the remainder and add them to the context
    //     3. add the remainder to (self.i_type, other.o_type), with replaced variables

    //     _

    // }

}

impl Restack {
    pub fn type_of(&self) -> Result<Type, RestackError> {
        let mut context = Context::new();
        let mut restack_type: Vec<TypeId> = (0..self.restack_depth).map(|x| TypeId { type_id: x }).collect();
        Ok(Type {
            context: context,
            i_type: restack_type.clone(),
            o_type: self.run(&mut restack_type)?,
        })
    }
}

/// Push(Elem),             // (t: type, elem: type(t)) : [] -> [ t ]
/// Restack(Restack),       // (r: restack) : [ .. ] -> [ .. ]
/// HashSha256,             // : [ bytes ] -> [ bytes ]
/// CheckLe,                // : [ x, x ] -> [ bool ]
/// CheckLt,                // : [ x, x ] -> [ bool ]
/// CheckEq,                // : [ x, x ] -> [ bool ]
/// Concat,                 // (t: type, prf: is_concat(t)) : [ t, t ] -> [ t ]
/// Slice,                  // (t: type, prf: is_slice(t)) : [ int, int, t ] -> [ t ]
/// Index,                  // (t: type, prf: is_index(t)) : [ int, t ] -> [ json ]
/// Lookup,                 // [ string, object ] -> [ json ]
/// AssertTrue,             // [ bool ] -> []
/// ToJson,                 // (t: type) : [ t ] -> [ json ]
/// UnpackJson(ElemSymbol), // (t: type) : [ json ] -> [ t ]
/// StringToBytes,          // [ string ] -> [ bytes ]
impl Instruction {
    pub fn type_of(&self) -> Result<Type, RestackError> {
        match self {
            Instruction::Restack(restack) => Ok(restack.type_of()?),

            Instruction::AssertTrue => {
                let mut context = Context::new();
                let bool_var = context.push(ElemSymbol::Bool.elem_type());
                Ok(Type {
                    context: context,
                    i_type: vec![bool_var],
                    o_type: vec![],
                })
            },

            Instruction::Push(elem) => {
                let mut context = Context::new();
                let elem_var = context.push(elem.elem_type());
                Ok(Type {
                    context: context,
                    i_type: vec![],
                    o_type: vec![elem_var],
                })
            },

            Instruction::HashSha256 => {
                let mut context = Context::new();
                let bytes_var = context.push(ElemSymbol::Bytes.elem_type());
                Ok(Type {
                    context: context,
                    i_type: vec![bytes_var],
                    o_type: vec![bytes_var],
                })
            },

            Instruction::ToJson => {
                let mut context = Context::new();
                let any_var = context.push(ElemType::any());
                let json_var = context.push(ElemSymbol::Json.elem_type());
                Ok(Type {
                    context: context,
                    i_type: vec![any_var],
                    o_type: vec![json_var],
                })
            },

            Instruction::StringToBytes => {
                let mut context = Context::new();
                let string_var = context.push(ElemSymbol::String.elem_type());
                let bytes_var = context.push(ElemSymbol::Bytes.elem_type());
                Ok(Type {
                    context: context,
                    i_type: vec![string_var],
                    o_type: vec![bytes_var],
                })
            },

            Instruction::UnpackJson(elem_symbol) => {
                let mut context = Context::new();
                let json_var = context.push(ElemSymbol::Json.elem_type());
                let elem_symbol_var = context.push(elem_symbol.elem_type());
                Ok(Type {
                    context: context,
                    i_type: vec![json_var],
                    o_type: vec![elem_symbol_var],
                })
            },

            Instruction::CheckLe => {
                let mut context = Context::new();
                let any_lhs_var = context.push(ElemType::any());
                let any_rhs_var = context.push(ElemType::any());
                let bool_var = context.push(ElemSymbol::Bool.elem_type());
                Ok(Type {
                    context: context,
                    i_type: vec![any_lhs_var, any_rhs_var],
                    o_type: vec![bool_var],
                })
            },

            Instruction::CheckLt => {
                let mut context = Context::new();
                let any_lhs_var = context.push(ElemType::any());
                let any_rhs_var = context.push(ElemType::any());
                let bool_var = context.push(ElemSymbol::Bool.elem_type());
                Ok(Type {
                    context: context,
                    i_type: vec![any_lhs_var, any_rhs_var],
                    o_type: vec![bool_var],
                })
            },

            Instruction::CheckEq => {
                let mut context = Context::new();
                let any_lhs_var = context.push(ElemType::any());
                let any_rhs_var = context.push(ElemType::any());
                let bool_var = context.push(ElemSymbol::Bool.elem_type());
                Ok(Type {
                    context: context,
                    i_type: vec![any_lhs_var, any_rhs_var],
                    o_type: vec![bool_var],
                })
            },

            Instruction::Concat => {
                let mut context = Context::new();
                let concat_var = context.push(ElemType::concat_type());
                Ok(Type {
                    context: context,
                    i_type: vec![concat_var, concat_var],
                    o_type: vec![concat_var],
                })
            },

            Instruction::Index => {
                let mut context = Context::new();
                let number_var = context.push(ElemSymbol::Number.elem_type());
                let index_var = context.push(ElemType::index_type());
                Ok(Type {
                    context: context,
                    i_type: vec![number_var, index_var],
                    o_type: vec![index_var],
                })
            },

            Instruction::Lookup => {
                let mut context = Context::new();
                let string_var = context.push(ElemSymbol::String.elem_type());
                let object_var = context.push(ElemSymbol::Object.elem_type());
                Ok(Type {
                    context: context,
                    i_type: vec![string_var, object_var],
                    o_type: vec![object_var],
                })
            },

            Instruction::Slice => {
                let mut context = Context::new();
                let offset_number_var = context.push(ElemSymbol::Number.elem_type());
                let length_number_var = context.push(ElemSymbol::Number.elem_type());
                let slice_var = context.push(ElemType::slice_type());
                Ok(Type {
                    context: context,
                    i_type: vec![offset_number_var, length_number_var, slice_var],
                    o_type: vec![slice_var],
                })
            },
        }
    }
}




// #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
// pub struct TyUnifyLocation {
//     lhs: SrcRange,
//     rhs: SrcRange,
//     stack_position: usize,
// }

// impl Ty {

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

// #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
// pub struct StackTy {
//     src_range: SrcRange,
//     in_type: Vec<Ty>,
//     out_type: Vec<Ty>,
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
//         let middle_ty = Self::unify_ty_sets(self.out_type, other.in_type, self.src_range, other.src_range);
//         let self_remainder = Self::diff_ty_sets(middle_ty, self.out_type);
//         let other_remainder = Self::diff_ty_sets(middle_ty, other.in_type);
//             in_type: self.in_type + self_remainder
//             out_type: other.out_type + other_remainder

//         StackTy {
//             src_range: self.src_range.append(other.src_range)?,
//             in_type Self::union_ty_sets(self.in_type, self_remainder),
//             out_type Self::union_ty_sets(other.out_type, other_remainder),
//         }
//     }

//     pub fn infer_instruction(instruction: &Instruction, src_location: usize) -> Self {
//         let instruction_ty_sets = instruction.ty_sets();
//         StackTy {
//             src_range: SrcRange::singleton(src_location),
//             in_type instruction_ty_sets.0,
//             out_type instruction_ty_sets.1,
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

