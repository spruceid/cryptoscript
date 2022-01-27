use generic_array::{typenum::U32, GenericArray};

#[derive(Debug, PartialEq)]
pub enum Elem {
    Bool(bool),
    Bytes32(GenericArray<u8, U32>),
    BytesN(Vec<u8>),
}

#[derive(Debug)]
pub enum Instruction {
    Push(Elem),
    // FnRestack(GenericArray<u8, U32>),
    FnHashSha256,
    FnCheckEqual,
    FnAssertTrue,
}

pub type Instructions = Vec<Instruction>;
