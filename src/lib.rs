#![warn(missing_docs, elided_lifetimes_in_paths, explicit_outlives_requirements, keyword_idents, missing_copy_implementations, missing_debug_implementations, non_ascii_idents, noop_method_call, single_use_lifetimes, trivial_casts, trivial_numeric_casts, unreachable_pub, unused_crate_dependencies, unused_extern_crates, unused_import_braces, unused_lifetimes, unused_qualifications)]

// #![warn(unused_results)]

#![deny(unsafe_code, unsafe_op_in_unsafe_fn)]

mod restack;
pub use restack::{Restack, StackIx};
mod arbitrary;
pub use arbitrary::{ArbitraryNumber, ArbitraryMap, ArbitraryValue};
mod elem;
pub use elem::{Elem, ElemSymbol};
mod elem_type;
pub use elem_type::{ElemType, StackType};
mod an_elem;
pub use an_elem::{AnElem, AnElemError};
mod location;
pub use location::{ArgumentIndex, LineNo};
mod stack;
pub use stack::{Stack, StackError};
mod types;
mod types_scratch;
pub use types_scratch::{AllElems, IOList};
mod json_template;
pub use json_template::{TMap, TValue, TValueRunError, Template};
mod query;
pub use query::{QueryTemplate, QueryTemplates, Query, QueryType, QueryError};
mod untyped_instruction;
pub use untyped_instruction::Instruction;
mod untyped_instructions;
pub use untyped_instructions::Instructions;
mod typed_instruction;
pub use typed_instruction::IsInstructionT;
mod typed_instructions;
pub use typed_instructions::{AssertTrue, Concat, Push, Lookup, UnpackJson, Index, CheckEq, BytesEq, StringEq, CheckLe, CheckLt, StringToBytes, ToJson, Slice, HashSha256};
mod typed_instr;
pub use typed_instr::Instr;
mod typed_instrs;
pub use typed_instrs::Instrs;
mod parse;
pub use parse::{parse, parse_json};

mod rest_api;
pub use rest_api::Api;
mod cli;
pub use cli::Cli;

use sha2::{Digest, Sha256};

// /**
//  * Types:
//  * - UnsignedInteger
//  * - Integer
//  * - Float64
//  * - Bytes(N)
//  * - Multibase
//  * - Multihash
//  * - Multiaddr
//  * - KeyTypes={Ed25519,Secp256k1,Secp256r1,Bls12_381}
//  * - PublicKey(KeyType)
//  * - PrivateKey(KeyType)
//  * - JWT
//  * - JWS
//  * - JWE
//  * - LDP
//  * - JSON
//  * - CBOR
//  *
//  * Functions
//  * - Sign :: Bytes(N) -> PrivateKey(KeyType) => Bytes(SignatureSize[KeyType])
//  * - VerifySignature :: Bytes(N) -> Bytes(SignatureSize[KeyType]) -> PublicKey(KeyType) => Boolean
//  * - VerifyRecoveredSignature :: Bytes(N) -> Bytes(SignatureSize[KeyType]) => Boolean
//  * - HashSha3_256 :: Bytes(N) => Bytes(32)
//  * - Equal
//  * - AssertTrue
//  *
//  * Example
//  *      push b"I am the walrus.";
//  *      hash_sha256;
//  *      push 0x475b03e74f7ee448273dbde5ab892746c7b23a2b4d050ccb7d9270b6fb152b72;
//  *      check_equal;
//  *      assert_true;
//  *
//  *  Example
//  *      setup {
//  *          push b"I am the walrus.";
//  *      }
//  *      challenge {
//  *          hash_sha256;
//  *          push 0x475b03e74f7ee448273dbde5ab892746c7b23a2b4d050ccb7d9270b6fb152b72;
//  *          check_equal;
//  *          assert_true;
//  *      }
//  */

fn sha256(input: &Vec<u8>) -> Vec<u8> {
    // create a Sha256 object
    let mut hasher = Sha256::new();

    // write input message
    hasher.update(input);

    // read hash digest and consume hasher
    let result = hasher.finalize();
    return result.to_vec();
}

#[cfg(test)]
mod tests {
    use super::*;
    use generic_array::{typenum::U32, GenericArray};
    use hex_literal::hex;
    use sha3::{Digest as Sha3_Digest, Sha3_256};

    fn sha3_256(input: &Vec<u8>) -> GenericArray<u8, U32> {
        // create a Sha256 object
        let mut hasher = Sha3_256::new();

        // write input message
        hasher.update(input);

        // read hash digest and consume hasher
        let result = hasher.finalize();
        return result;
    }

    fn drop_bytes(n: usize, input: &Vec<u8>) -> Vec<u8> {
        let mut result = input.clone();
        result.drain(..n);
        return result;
    }

    #[test]
    fn test_sha2() {
        let result = sha256(&b"hello world".to_vec());
        assert_eq!(
            result[..],
            hex!(
                "
            b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9
        "
            )[..]
        );
    }

    #[test]
    fn test_sha3() {
        let result = sha3_256(&b"hello world".to_vec());
        // println!("{:x?}", hex_encode(result.as_slice()));
        assert_eq!(
            result[..],
            hex!(
                "
            644bcc7e564373040999aac89e7622f3ca71fba1d972fd94a31c3bfbf24e3938
        "
            )[..]
        );
    }

    #[test]
    fn test_drop_bytes() {
        let result = drop_bytes(6, &b"hello world".to_vec());
        assert_eq!(&result[..], b"world");
    }
}
