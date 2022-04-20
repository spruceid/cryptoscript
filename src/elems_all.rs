use crate::elem::Elem;
use crate::elems_singleton::Singleton;
use crate::elems_or::Or;

use std::fmt::Debug;

use serde_json::{Map, Number, Value};
use generic_array::functional::FunctionalSequence;
use generic_array::{GenericArray, ArrayLength};


// TODO: AnElem: &self -> AllElems<U1>
/// All possible Elem types, encoded using Or and Singleton.
pub type AllElems<N> =
    Or<(), N,
    Or<bool, N,
    Or<Number, N,
    Or<Vec<u8>, N,
    Or<String, N,
    Or<Vec<Value>, N,
    Or<Map<String, Value>, N,
    Singleton<Value, N>>>>>>>>;

impl<N> AllElems<N>
where
    N: Debug +
    ArrayLength<()> +
    ArrayLength<bool> +
    ArrayLength<Number> +
    ArrayLength<Vec<u8>> +
    ArrayLength<String> +
    ArrayLength<Vec<Value>> +
    ArrayLength<Map<String, Value>> +
    ArrayLength<Value> +
    ArrayLength<Elem>,
{
    /// Untype AllElems to Elem
    pub fn untyped(&self) -> GenericArray<Elem, N> {
        match self {
            Or::Left(array) => {
                array.map(|_x| Elem::Unit)
            },
            Or::Right(Or::Left(array)) => {
                array.map(|&x| Elem::Bool(x))
            },
            Or::Right(Or::Right(Or::Left(array))) => {
                array.map(|x| Elem::Number(x.clone()))
            },
            Or::Right(Or::Right(Or::Right(Or::Left(array)))) => {
                array.map(|x| Elem::Bytes(x.clone()))
            },
            Or::Right(Or::Right(Or::Right(Or::Right(Or::Left(array))))) => {
                array.map(|x| Elem::String(x.clone()))
            },
            Or::Right(Or::Right(Or::Right(Or::Right(Or::Right(Or::Left(array)))))) => {
                array.map(|x| Elem::Array(x.clone()))
            },
            Or::Right(Or::Right(Or::Right(Or::Right(Or::Right(Or::Right(Or::Left(array))))))) => {
                array.map(|x| Elem::Object(x.clone()))
            },
            Or::Right(Or::Right(Or::Right(Or::Right(Or::Right(Or::Right(Or::Right(Singleton { array }))))))) => {
                array.map(|x| Elem::Json(x.clone()))
            },
        }
    }
}

