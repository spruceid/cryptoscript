// use crate::elem::{Elem, ElemType, ElemTypeError, ElemSymbol, StackType, AnElem};
// use crate::stack::{Stack, StackError};
// use crate::restack::{Restack, RestackError};
// use crate::types::{Context, ContextError, Type, Empty, AnError, Nil};

use std::fmt;
// use std::fmt::Debug;
use std::sync::{Arc};
use std::marker::PhantomData;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{Visitor, MapAccess};

use indexmap::{IndexMap};
use serde_json::{Map, Number, Value};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TMap<T> {
    map: IndexMap<String, T>,
}

impl<T> TMap<T> {
    pub fn new() -> Self {
        TMap {
            map: IndexMap::new(),
        }
    }

    pub fn insert(&mut self, key: String, value: T) -> Option<T> {
        self.map.insert(key, value)
    }
}

impl Serialize for TMap<TValue> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.map.clone()
            .into_iter()
            .map(|(x, y)| Ok((x, y.to_json()?)))
            .collect::<Result<Map<String, Value>, TValueError>>()
            .map_err(|e| serde::ser::Error::custom(format!("Serialize for TMap<TValue>:\n{:?}", e)))?
            .serialize(serializer)
    }
}

struct TMapVisitor<T> {
    marker: PhantomData<fn() -> TMap<T>>
}

impl<T> TMapVisitor<T> {
    fn new() -> Self {
        TMapVisitor {
            marker: PhantomData
        }
    }
}

impl<'de, T> Visitor<'de> for TMapVisitor<T>
where
    T: Deserialize<'de>,
{
    type Value = TMap<T>;

    // Format a message stating what data this Visitor expects to receive.
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        // TODO: extend description
        formatter.write_str("TMap")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut map = IndexMap::with_capacity(access.size_hint().unwrap_or(0));

        while let Some((key, value)) = access.next_entry()? {
            map.insert(key, value);
        }

        Ok(TMap {
            map: map,
        })
    }
}

impl<'de, T> Deserialize<'de> for TMap<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(TMapVisitor::new())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TValue {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<TValue>),
    Object(TMap<TValue>),
    Var(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TValueRunError {
    variable: String,
    value: Vec<TValue>,
    variables: Map<String, Value>,
}

impl TValue {
    pub fn from_json(json: Value) -> Self {
        match json {
            Value::Null => Self::Null,
            Value::Bool(x) => Self::Bool(x),
            Value::Number(x) => Self::Number(x),
            Value::String(x) => Self::String(x),
            Value::Array(x) => Self::Array(x.into_iter().map(|x| TValue::from_json(x)).collect()),
            Value::Object(x) => Self::Object(TMap {
                map: x.into_iter().map(|(x, y)| (x, TValue::from_json(y))).collect()
            }),
        }
    }

    pub fn to_json(&self) -> Result<Value, TValueError> {
        serde_json::to_value(self)
            .map_err(|e| TValueError::SerdeJsonError(Arc::new(e)))
    }

    /// Resolve all of the (Var)'s using the given (variables)
    pub fn run(self, variables: Map<String, Value>) -> Result<Value, TValueRunError> {
        let self_copy = self.clone();
        match self {
            Self::Null => Ok(Value::Null),
            Self::Bool(x) => Ok(Value::Bool(x)),
            Self::Number(x) => Ok(Value::Number(x)),
            Self::String(x) => Ok(Value::String(x)),
            Self::Array(x) => Ok(Value::Array(x.into_iter().map(|y| y.run(variables.clone())).collect::<Result<Vec<Value>, TValueRunError>>()?)),
            Self::Object(x) => Ok(Value::Object(x.map.into_iter().map(|(y, z)| Ok((y, z.run(variables.clone())?))).collect::<Result<Map<String, Value>, TValueRunError>>()?)),
            Self::Var(x) => {
                variables.get(&x)
                    .map(|y| y.clone())
                    .ok_or_else(|| TValueRunError {
                    variable: x,
                    value: vec![self_copy],
                    variables: variables,
                })
            },
        }
    }
}

#[derive(Clone, Debug, Error)]
pub enum TValueError {
    #[error("TValue::to_json:\n{0}")]
    SerdeJsonError(Arc<serde_json::Error>),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Template {
    // TODO: use impl instead of pub
    pub variables: Map<String, Value>,
    pub template: TValue,
}

impl Template {
    pub fn from_json(json: Value) -> Self {
        Template {
            variables: Map::new(),
            template: TValue::from_json(json),
        }
    }

    pub fn run(self) -> Result<Value, TValueRunError> {
        self.template.run(self.variables)
    }
}

