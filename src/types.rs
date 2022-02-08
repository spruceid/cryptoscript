use generic_array::{typenum::U32, GenericArray};
use thiserror::Error;

use std::cmp;


// extern crate base64;
use base64;
use serde::{de, Deserialize, Serialize};
// use serde_json::Result;

use serde::de::{Deserializer};
use serde::ser::{Serializer};


// TODO:
// - restack:
//     + DONE: add common stack manipulation constructors
//     + test against common stack manipulations, e.g. swap; swap = id

// - json
//     + add to stack type
//     + add construction/destruction primitives
//     + add property based tests



fn serialize_generic_array_u8_u32<T, S>(v: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: AsRef<[u8]>,
    S: Serializer,
{
    serializer.serialize_str(&base64::encode(v.as_ref()))
}

pub fn deserialize_generic_array_u8_u32<'de, D>(deserializer: D) -> Result<GenericArray<u8, U32>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    String::deserialize(deserializer)
        .and_then(|string| base64::decode(&string).map_err(|err| Error::custom(err.to_string())))
        .and_then(|vec| {
            GenericArray::from_exact_iter(vec).ok_or(
                de::Error::custom("String::deserialize failed to produce an array of length 32"))
        })
}


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Elem {
    Bool(bool),
    #[serde(serialize_with = "serialize_generic_array_u8_u32", deserialize_with = "deserialize_generic_array_u8_u32")]
    Bytes32(GenericArray<u8, U32>),
    BytesN(Vec<u8>),
}

#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Restack {
    restack_depth: StackIx,
    restack_vec: Vec<StackIx>,
}

impl Restack {
    // identity
    pub fn id() -> Self {
        Restack {
            restack_depth: 0,
            restack_vec: vec![],
        }
    }

    // swap first two stack elements
    pub fn swap() -> Self {
        Restack {
            restack_depth: 2,
            restack_vec: vec![1usize, 0],
        }
    }

    // drop the first (n) stack elements
    pub fn drop_n(n: usize) -> Self {
        Restack {
            restack_depth: n,
            restack_vec: vec![]
        }
    }

    // drop the first stack element
    pub fn drop() -> Self {
        Self::drop_n(1)
    }

    // duplicates the (ix)th value onto the top of the stack (0-indexed)
    pub fn dup_n(ix: usize) -> Self {
        Restack {
            restack_depth: ix+1,
            restack_vec: (ix..ix).chain(0..ix).collect(),
        }
    }

    // duplicates the 0th value onto the top of the stack (0-indexed)
    pub fn dup() -> Self {
        Self::dup_n(0)
    }

    // pull the (ix)th element to the top of the stack
    // dig 4 = { 5, [3, 0, 1, 2] }
    pub fn dig(ix: usize) -> Self {
        Restack {
            restack_depth: ix+1,
            restack_vec: (0..ix+1).cycle().skip(ix).take(ix+1).collect(),
        }
    }

    // push the top of the stack to the (ix)th position
    // dug 4 = { 5, [1, 2, 3, 0] }
    pub fn dug(ix: usize) -> Self {
        Restack {
            restack_depth: ix+1,
            restack_vec: (1..ix+1).chain(std::iter::once(0)).collect()
        }
    }

    // restack a Stack
    pub fn run(&self, stack: &mut Stack) -> Result<Stack, RestackError> {
        if self.restack_depth <= stack.len() {
            let result = self.restack_vec.iter().map(|&restack_index|
                match stack.get(restack_index) {
                    None => Err(RestackError::StackIndexInvalid{ restack_index: restack_index, restack_depth: self.restack_depth, }),
                    Some(stack_element) => Ok( stack_element.clone() ),
                }
            ).collect::<Result<Stack, RestackError>>();
            match result {
                Ok(mut result_ok) => {
                    result_ok.extend(stack.drain(self.restack_depth..));
                    Ok(result_ok) },
                Err(e) => Err(e)
            }

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

    // NOTE: unchecked (run valid_depth on arguments for safe version)
    // x.append(y).run(s) == x.run(y.run(s))
    pub fn append(&self, other: Self) -> Self {
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


#[derive(Debug, PartialEq, Error)]
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_restack_id() {
        let mut example_stack = vec![Elem::Bool(false), Elem::Bool(true)];
        let restack = Restack::id();
        assert!(restack.valid_depth(), "Restack::id() has invalid depth");
        assert_eq!(Ok(example_stack.clone()), restack.run(&mut example_stack))
    }

    #[test]
    fn test_restack_dig() {
        assert!(Restack::dig(4).valid_depth(), "Restack::dig(4) has invalid depth");
        assert_eq!(Restack { restack_depth: 5, restack_vec: vec![4, 0, 1, 2, 3] }, Restack::dig(4));
        let mut example_stack_in = vec![Elem::Bool(false), Elem::Bool(false), Elem::Bool(false), Elem::Bool(false), Elem::Bool(true)];
        let example_stack_out = vec![Elem::Bool(true), Elem::Bool(false), Elem::Bool(false), Elem::Bool(false), Elem::Bool(false)];
        assert_eq!(Ok(example_stack_out.clone()), Restack::dig(4).run(&mut example_stack_in))
    }

    #[test]
    fn test_restack_dug() {
        assert!(Restack::dug(4).valid_depth(), "Restack::dug(4) has invalid depth");
        assert_eq!(Restack { restack_depth: 5, restack_vec: vec![1, 2, 3, 4, 0] }, Restack::dug(4));
        let mut example_stack_in = vec![Elem::Bool(true), Elem::Bool(false), Elem::Bool(false), Elem::Bool(false), Elem::Bool(false)];
        let example_stack_out = vec![Elem::Bool(false), Elem::Bool(false), Elem::Bool(false), Elem::Bool(false), Elem::Bool(true)];
        assert_eq!(Ok(example_stack_out.clone()), Restack::dug(4).run(&mut example_stack_in))
    }

    #[test]
    fn test_restack_drop_n() {
        let example_stack_in = vec![Elem::Bool(false), Elem::Bool(true), Elem::Bool(false)];
        for example_stack_out in
            [vec![Elem::Bool(false), Elem::Bool(true), Elem::Bool(false)],
            vec![Elem::Bool(true), Elem::Bool(false)],
            vec![Elem::Bool(false)],
            vec![]] {
                let restack = Restack::drop_n(3 - example_stack_out.len());
                assert!(restack.valid_depth(), "Restack::drop_n(_) has invalid depth");
                assert_eq!(Ok(example_stack_out), restack.run(&mut example_stack_in.clone()));
        }
    }

    #[test]
    fn test_restack_drop() {
        let mut example_stack_in = vec![Elem::Bool(false), Elem::Bool(true)];
        let example_stack_out = vec![Elem::Bool(true)];
        let restack = Restack::drop();
        assert!(restack.valid_depth(), "Restack::drop() has invalid depth");
        assert_eq!(Ok(example_stack_out), restack.run(&mut example_stack_in))
    }

    #[test]
    fn test_restack_swap() {
        let mut example_stack_in = vec![Elem::Bool(false), Elem::Bool(true)];
        let example_stack_out = vec![Elem::Bool(true), Elem::Bool(false)];
        let restack = Restack::swap();
        assert!(restack.valid_depth(), "Restack::swap() has invalid depth");
        assert_eq!(Ok(example_stack_out), restack.run(&mut example_stack_in))
    }

    #[test]
    fn test_restack_swap_twice_append() {
        let mut example_stack = vec![Elem::Bool(false), Elem::Bool(true)];
        let restack = Restack::swap().append(Restack::swap());
        assert!(restack.valid_depth(), "Restack::swap().append(Restack::swap()) has invalid depth");
        assert_eq!(Ok(example_stack.clone()), restack.run(&mut example_stack))
    }

}

