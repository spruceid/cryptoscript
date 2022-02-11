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

use thiserror::Error;


pub fn parse_json(input: &str) -> Result<Instructions, ParseError> {
    match serde_json::from_str(&input) {
        Err(serde_error) => Err(ParseError::SerdeJsonError(serde_error)),
        Ok(instructions) => Ok(instructions),
    }
}

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
            Self::Unit => "Unit",
            Self::Bool(_) => "Bool",
            Self::Bytes(_) => "Bytes",
            Self::Number(_) => "Number",
            Self::String(_) => "String",
            Self::Array(_) => "Array",
            Self::Object(_) => "Object",
            Self::Json(_) => "JSON",
        }
    }
}

impl FromStr for Elem {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.as_bytes() {
            [b'b', b'"', inner @ .., b'"'] => {
                return Ok(Elem::Bytes(inner.to_vec()));
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

                return Ok(Elem::Bytes(bytes))
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
    #[error("error from serde_json ({0})")]
    SerdeJsonError(serde_json::Error),
}

