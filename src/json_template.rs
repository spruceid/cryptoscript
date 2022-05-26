use std::fmt::{self, Formatter};
use std::sync::Arc;
use std::marker::PhantomData;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{Visitor, MapAccess};

use indexmap::IndexMap;
use serde_json::{Map, Number, Value};
use thiserror::Error;

/// Map<String, T> defined to be convenient to Serialize and Deserialize
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TMap<T> {
    map: IndexMap<String, T>,
}

impl<T> TMap<T> {
    /// IndexMap::new
    pub fn new() -> Self {
        TMap {
            map: IndexMap::new(),
        }
    }

    /// IndexMap::insert
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
    fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
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

/// serde_json::Value with Var's
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TValue {
    /// serde_json::Null
    Null,

    /// serde_json::Bool
    Bool(bool),

    /// serde_json::Number
    Number(Number),

    /// serde_json::String
    String(String),

    /// serde_json::Array with Var's
    Array(Vec<TValue>),

    /// serde_json::Object with Var's
    Object(TMap<TValue>),

    /// Named variable. See TValue::run for more detail
    Var(String),
}

/// An error encountered during the execution of TValue::run
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TValueRunError {
    variable: String,
    value: Vec<TValue>,
    variables: Map<String, Value>,
}

impl TValue {
    /// Convert from JSON, ignoring Var's
    ///
    /// Use Deserialize to convert including Var's
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

    /// Convert to JSON using derived Serialize instance
    pub fn to_json(&self) -> Result<Value, TValueError> {
        serde_json::to_value(self)
            .map_err(|e| TValueError::SerdeJsonError(Arc::new(e)))
    }

    /// Resolve all of the (Var)'s using the given variables.
    ///
    /// For example, if the Map includes the association ("foo", "bar"),
    /// all occurences of Var("foo") will be replaced with "bar".
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

/// A template that inclues an associated set of variables
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Template {
    /// Set of variables to resolve on the TValue
    variables: Map<String, Value>,

    /// Template value
    template: TValue,
}

impl Template {
    /// New template with an empty set of variables
    pub fn new(template: TValue) -> Self {
        Self {
            variables: Map::new(),
            template: template,
        }
    }

    /// Set the given variable name to the given Value
    pub fn set(&mut self, name: String, value: Value) -> () {
        self.variables.insert(name, value);
    }

    /// Deserialize the Template from JSON and instantiate an empty set of variables
    pub fn from_json(json: Value) -> Self {
        Self::new(TValue::from_json(json))
    }

    /// Run the TValue given the provided variables
    pub fn run(self) -> Result<Value, TValueRunError> {
        self.template.run(self.variables)
    }
}
