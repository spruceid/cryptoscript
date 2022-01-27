/// Parser for the cryptoscript language.
///
/// Partial definition to cover the terms in the example in lib.rs.
///   TERM -> push PUSH_VALUE | FUNCTION
///   TERMS -> TERM ; TERMS | TERM ;
///   FUNCTION -> hash_sha256 | check_equal | assert_true
///   PUSH_VALUE -> b"CHARS" | 0xHEX
///
/// Where CHARS is any number of characters which aren't escaped double-quotes (\") and HEX is a 64
/// digit hexadecimal number.

use crate::types::{Elem, Instruction, Instructions};

use std::str::FromStr;

use generic_array::{typenum::U32, GenericArray};
use thiserror::Error;

pub fn parse(input: &str) -> Result<Instructions, ParseError> {
    input
        .split(';')
        .map(|term| term.trim())
        .filter(|&term| !term.is_empty())
        .map(|term| parse_instruction(term))
        .collect()
}

fn parse_instruction(term: &str) -> Result<Instruction, ParseError> {
    if let Some(rest) = term.strip_prefix("push") {
        return Ok(Instruction::Push(rest.trim().parse()?));
    }
    match term {
        "assert_true" => Ok(Instruction::FnAssertTrue),
        "check_equal" => Ok(Instruction::FnCheckEqual),
        "hash_sha256" => Ok(Instruction::FnHashSha256),
        _ => Err(ParseError::UnsupportedInstruction(term.to_string())),
    }
}

impl Elem {
    pub fn simple_type(&self) -> &'static str {
        match self {
            Self::Bool(_) => "bool",
            Self::Bytes32(_) => "Bytes(32)",
            Self::BytesN(_) => "Bytes(N)",
        }
    }
}

impl FromStr for Elem {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.as_bytes() {
            [b'b', b'"', inner @ .., b'"'] => {
                return Ok(Elem::BytesN(inner.to_vec()));
            }
            [b'0', b'x', hex_digits @ ..] => {
                if hex_digits.len() != 64 {
                    return Err(ParseError::HexElemWrongLength(hex_digits.len()));
                }
                // Second value can be ignored since there is a check above for evenness.
                let (bytes, _) = hex_digits
                    .iter()
                    // convert the hex digits to their decimal value
                    .map(|byte| match byte {
                        // convert digits
                        digit @ 48..=57 => Ok(digit - 48),
                        // convert uppercase A-F
                        upper @ 65..=70 => Ok(upper - 55),
                        // convert lowercase a-f
                        lower @ 97..=102 => Ok(lower - 87),
                        invalid => Err(ParseError::HexElemInvalid(*invalid as char)),
                    })
                    // pair up the hex digits to make bytes
                    .try_fold(
                        (vec![], None),
                        |(mut acc, previous), digit| match previous {
                            None => Ok((acc, Some(digit?))),
                            Some(top) => {
                                acc.push(top * 16 + digit?);
                                Ok((acc, None))
                            }
                        },
                    )?;

                if let Some(array) = GenericArray::from_exact_iter(bytes) {
                    return Ok(Elem::Bytes32(array));
                } else {
                    use std::hint::unreachable_unchecked;
                    // if the 'bytes' vec has been constructed without error, then it is 32 bytes
                    // long, as the hex_digits slice is checked to be 64 digits long, and each pair
                    // of digits is used to make one byte.
                    unsafe { unreachable_unchecked() }
                }
            }
            // No need to support booleans, but it is trivial to do so.
            _ => Err(ParseError::UnsupportedElem(s.to_string())),
        }
    }
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("input hex number did not have the expected 64 digits ({0})")]
    HexElemWrongLength(usize),
    #[error("there is an invalid character in the hex number ({0})")]
    HexElemInvalid(char),
    #[error("elem is malformed or cannot be parsed in this context")]
    UnsupportedElem(String),
    #[error("instruction is malformed or cannot be parsed in this context")]
    UnsupportedInstruction(String),
}
