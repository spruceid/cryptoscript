use generic_array::{typenum::U32, GenericArray};
use thiserror::Error;
use std::cmp;

#[derive(Clone, Debug, PartialEq)]
pub enum Elem {
    Bool(bool),
    Bytes32(GenericArray<u8, U32>),
    BytesN(Vec<u8>),
}

#[derive(Debug)]
pub enum Instruction {
    Push(Elem),
    FnRestack(Restack),
    FnHashSha256,
    FnCheckEqual,
    FnAssertTrue,
}


pub type StackIx = usize;
pub type Stack = Vec<Elem>;

// Stack manipulation:
// - All stack manipulations:
//     + dig
//     + dug
//     + dip
//     + dup
//     + swap
//     + drop
// - they all boil down to:
//     1. drop inputs
//     2. replicate inputs
//     3. reorder inputs
// - which conveniently boils down to:
//     + xs : [ old_stack_index ]
//     + map (\x -> xs !! x) xs
// - successful iff all old_stack_index's < length stack
// - pretty-printing?
//     + REQUIRED: constant compile-time choice of manipulations
//     + local: just print [x_old_stack_index_0, x_old_stack_index_1, ..]
//     + global: keep track of stack indices (always possible?) and print where it's from???
#[derive(Debug, PartialEq)]
pub struct Restack {
    restack_depth: StackIx,
    restack_vec: Vec<StackIx>
}

impl Restack {
    pub fn run(&self, stack: &Stack) -> Result<Stack, RestackError> {
        if self.restack_depth <= stack.len() {
            self.restack_vec.iter().map(|&restack_index|
                match stack.get(restack_index) {
                    None => Err(RestackError::StackIndexInvalid{ restack_index: restack_index, restack_depth: self.restack_depth, }),
                    Some(stack_element) => Ok( stack_element.clone() ),
                }
            ).collect::<Result<Stack, RestackError>>()
        } else {
            Err(RestackError::InvalidDepth{ stack_len: stack.len(), restack_depth: self.restack_depth, })
        }
    }

    // self.valid_depth() ->
    // self.restack_depth <= xs.len() ->
    // self.run(xs).is_ok() == true
    pub fn valid_depth(&self) -> bool {
        !self.restack_vec.iter().any(|&restack_index| self.restack_depth <= restack_index)
    }

    // x.append(y).run(s) == x.run(y.run(s))
    pub fn append(&self, other: Restack) -> Restack {
        Restack {
            restack_depth: cmp::max(self.restack_depth, other.restack_depth),
            restack_vec: self.restack_vec.iter().map(|&restack_index|
                match other.restack_vec.get(restack_index) {
                    None => restack_index,
                    Some(stack_index) => stack_index.clone(),
                }
            ).collect()
        }
    }

}


#[derive(Debug, Error)]
pub enum RestackError {
    #[error("invalid Restack: restack_index = {restack_index:?} out of bounds for restack_depth = {restack_depth:?}")]
    StackIndexInvalid {
        restack_index: usize,
        restack_depth: usize,
    },
    #[error("attempt to restack {restack_depth:?} elements of a stack with only {stack_len:?} elements")]
    InvalidDepth {
        stack_len: usize,
        restack_depth: usize,
    },
}


pub type Instructions = Vec<Instruction>;


