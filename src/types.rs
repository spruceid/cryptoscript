use crate::restack::Restack;
use crate::elem::{Elem, ElemSymbol};

use enumset::{EnumSet};
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
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Context {
    context: Vec<ElemType>
}

impl Context {
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TypeId {
    type_id: usize,
}

impl Context {
    pub fn new() -> Self {
        Context {
            context: vec![],
        }
    }

    pub fn size(&self) -> usize {
        self.context.len()
    }

    pub fn push(&mut self, elem_type: ElemType) -> TypeId {
        let push_id = TypeId {
            type_id: self.size(),
        };
        self.context.push(elem_type);
        push_id
    }
}


#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Type {
    context: Context,
    i_type: Vec<TypeId>,
    o_type: Vec<TypeId>,
}

impl Type {
    pub fn is_valid(&self) -> bool {
        let context_size = self.context.size();
        !(self.i_type.iter().any(|x| x.type_id >= context_size) ||
          self.o_type.iter().any(|x| x.type_id >= context_size))
    }
}

// TODO: implement
impl Restack {
    pub fn type_of(&self) -> Type {
        panic!("Restack.type_of unimplemented");

        let mut context = Context::new();
        // let bool_var = context.push(ElemSymbol::Bool.elem_type());
        Type {
            context: context,
            i_type: vec![],
            o_type: vec![],
        }
    }
}

impl Instruction {
    pub fn type_of(&self) -> Type {
        match self {
            Instruction::Restack(restack) => restack.type_of(),

            Instruction::AssertTrue => {
                let mut context = Context::new();
                let bool_var = context.push(ElemSymbol::Bool.elem_type());
                Type {
                    context: context,
                    i_type: vec![bool_var],
                    o_type: vec![],
                }
            },

            Instruction::Push(elem) => {
                let mut context = Context::new();
                let elem_var = context.push(elem.elem_type());
                Type {
                    context: context,
                    i_type: vec![],
                    o_type: vec![elem_var],
                }
            },

            Instruction::HashSha256 => {
                let mut context = Context::new();
                let bytes_var = context.push(ElemSymbol::Bytes.elem_type());
                Type {
                    context: context,
                    i_type: vec![bytes_var],
                    o_type: vec![bytes_var],
                }
            },

            Instruction::ToJson => {
                let mut context = Context::new();
                let any_var = context.push(ElemType::any());
                let json_var = context.push(ElemSymbol::Json.elem_type());
                Type {
                    context: context,
                    i_type: vec![any_var],
                    o_type: vec![json_var],
                }
            },

            Instruction::StringToBytes => {
                let mut context = Context::new();
                let string_var = context.push(ElemSymbol::String.elem_type());
                let bytes_var = context.push(ElemSymbol::Bytes.elem_type());
                Type {
                    context: context,
                    i_type: vec![string_var],
                    o_type: vec![bytes_var],
                }
            },

            Instruction::UnpackJson(elem_symbol) => {
                let mut context = Context::new();
                let json_var = context.push(ElemSymbol::Json.elem_type());
                let elem_symbol_var = context.push(elem_symbol.elem_type());
                Type {
                    context: context,
                    i_type: vec![json_var],
                    o_type: vec![elem_symbol_var],
                }
            },

            Instruction::CheckLe => {
                let mut context = Context::new();
                let any_lhs_var = context.push(ElemType::any());
                let any_rhs_var = context.push(ElemType::any());
                let bool_var = context.push(ElemSymbol::Bool.elem_type());
                Type {
                    context: context,
                    i_type: vec![any_lhs_var, any_rhs_var],
                    o_type: vec![bool_var],
                }
            },

            Instruction::CheckLt => {
                let mut context = Context::new();
                let any_lhs_var = context.push(ElemType::any());
                let any_rhs_var = context.push(ElemType::any());
                let bool_var = context.push(ElemSymbol::Bool.elem_type());
                Type {
                    context: context,
                    i_type: vec![any_lhs_var, any_rhs_var],
                    o_type: vec![bool_var],
                }
            },

            Instruction::CheckEq => {
                let mut context = Context::new();
                let any_lhs_var = context.push(ElemType::any());
                let any_rhs_var = context.push(ElemType::any());
                let bool_var = context.push(ElemSymbol::Bool.elem_type());
                Type {
                    context: context,
                    i_type: vec![any_lhs_var, any_rhs_var],
                    o_type: vec![bool_var],
                }
            },

            Instruction::Concat => {
                let mut context = Context::new();
                let concat_var = context.push(ElemType::concat_type());
                Type {
                    context: context,
                    i_type: vec![concat_var, concat_var],
                    o_type: vec![concat_var],
                }
            },

            Instruction::Index => {
                let mut context = Context::new();
                let number_var = context.push(ElemSymbol::Number.elem_type());
                let index_var = context.push(ElemType::index_type());
                Type {
                    context: context,
                    i_type: vec![number_var, index_var],
                    o_type: vec![index_var],
                }
            },

            Instruction::Lookup => {
                let mut context = Context::new();
                let string_var = context.push(ElemSymbol::String.elem_type());
                let object_var = context.push(ElemSymbol::Object.elem_type());
                Type {
                    context: context,
                    i_type: vec![string_var, object_var],
                    o_type: vec![object_var],
                }
            },

            Instruction::Slice => {
                let mut context = Context::new();
                let offset_number_var = context.push(ElemSymbol::Number.elem_type());
                let length_number_var = context.push(ElemSymbol::Number.elem_type());
                let slice_var = context.push(ElemType::slice_type());
                Type {
                    context: context,
                    i_type: vec![offset_number_var, length_number_var, slice_var],
                    o_type: vec![slice_var],
                }
            },
        }
    }
}

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




// // TODO: use in_ty/out_ty

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

