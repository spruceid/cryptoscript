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
//     + add common stack manipulation constructors
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
    pub fn id() -> Self {
        Restack {
            restack_depth: 0,
            restack_vec: vec![],
        }
    }

    pub fn swap() -> Self {
        Restack {
            restack_depth: 2,
            restack_vec: vec![1usize, 0],
        }
    }


    pub fn run(&self, stack: &Stack) -> Result<Stack, RestackError> {
        if self.restack_depth <= stack.len() {
            self.restack_vec.iter().map(|&restack_index|
                match stack.get(restack_index) {
                    None => Err(RestackError::StackIndexInvalid{ restack_index: restack_index, restack_depth: self.restack_depth, }),
                    Some(stack_element) => Ok( stack_element.clone() ),
                }
            ).collect::<Result<Stack, RestackError>>()?.extend(stack.drain(self.restack_depth..))

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
    fn test_id_restack() {
        let example_stack = vec![Elem::Bool(false), Elem::Bool(true)];
        assert_eq!(Ok(example_stack.clone()), Restack::id().run(&example_stack))

    }

    // #[test]
    // fn test_sha2() {
    //     let result = sha256(&b"hello world".to_vec());
    //     assert_eq!(
    //         result[..],
    //         hex!(
    //             "
    //         b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9
    //     "
    //         )[..]
    //     );
    // }

    // #[test]
    // fn test_sha3() {
    //     let result = sha3_256(&b"hello world".to_vec());
    //     // println!("{:x?}", hex_encode(result.as_slice()));
    //     assert_eq!(
    //         result[..],
    //         hex!(
    //             "
    //         644bcc7e564373040999aac89e7622f3ca71fba1d972fd94a31c3bfbf24e3938
    //     "
    //         )[..]
    //     );
    // }

    // #[test]
    // fn test_drop_bytes() {
    //     let result = drop_bytes(6, &b"hello world".to_vec());
    //     assert_eq!(&result[..], b"world");
    // }

}


