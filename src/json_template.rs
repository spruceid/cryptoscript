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

//// BEGIN query
use reqwest::{Client, Response};
use std::fs;
use std::path::PathBuf;

use tokio_stream::{self as stream, StreamExt};
//// END query


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









//// BEGIN query
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryType {
    Get,
    Put,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Query {
    pub name: String,
    pub url: String,
    pub template: TValue,
    pub cached: bool,
    pub query_type: QueryType,
}

#[derive(Clone, Debug, Error)]
pub enum QueryError {
    #[error("Query::get_cached: not cached:\n{name:?}\n{url:?}")]
    NotCached {
        name: String,
        url: String,
    },

    #[error("Query::run: request failed:\nresponse:\n{response}")]
    RequestFailed {
        response: String,
        // response: Arc<reqwest::Response>,
        // body: String,
    },

    #[error("TValueRunError:\n{0:?}")]
    TValueRunError(TValueRunError),

    #[error("ReqwestError:\n{0}")]
    ReqwestError(Arc<reqwest::Error>),

    #[error("StdIoError:\n{0}")]
    StdIoError(Arc<std::io::Error>),

    #[error("SerdeJsonError:\n{0}")]
    SerdeJsonError(Arc<serde_json::Error>),
}

impl From<TValueRunError> for QueryError {
    fn from(error: TValueRunError) -> Self {
        Self::TValueRunError(error)
    }
}

impl From<reqwest::Error> for QueryError {
    fn from(error: reqwest::Error) -> Self {
        Self::ReqwestError(Arc::new(error))
    }
}

impl From<std::io::Error> for QueryError {
    fn from(error: std::io::Error) -> Self {
        Self::StdIoError(Arc::new(error))
    }
}

impl From<serde_json::Error> for QueryError {
    fn from(error: serde_json::Error) -> Self {
        Self::SerdeJsonError(Arc::new(error))
    }
}

impl Query {
    pub fn to_json(&self) -> Result<Value, QueryError> {
        Ok(serde_json::to_value(self)?)
    }

    pub async fn get_cached(&self, variables: Map<String, Value>, cache_location: PathBuf) -> Result<Value, QueryError> {
        if self.cached {
            println!("Checking cache: {:?}", cache_location.clone());
            let cache_str = fs::read_to_string(cache_location)?;
            let cache: Map<String, Value> = serde_json::from_str(&cache_str)?;
            let cache_index = format!("{:?}:{:?}", self.name, variables);
            cache.get(&cache_index).ok_or_else(|| {
                QueryError::NotCached {
                    name: self.name.clone(),
                    url: self.url.clone(),
            }}).map(|x| x.clone())
        } else {
            Err(QueryError::NotCached {
                name: self.name.clone(),
                url: self.url.clone(),
            })
        }
    }

    pub async fn put_cached(&self, result: Value, variables: Map<String, Value>, cache_location: PathBuf) -> Result<(), QueryError> {
        if self.cached {
            // TODO: don't re-add to cache if fetched from get_cached
            println!("Adding to cache: {:?}", cache_location.clone());
            let mut cache: Map<String, Value> = if cache_location.as_path().exists() {
                let cache_str = fs::read_to_string(cache_location.clone())?;
                serde_json::from_str(&cache_str)?
            } else {
                Map::new()
            };
            let cache_index = format!("{:?}:{:?}", self.name, variables);
            cache.insert(cache_index, result);
            let cache_json = serde_json::to_string_pretty(&serde_json::to_value(cache).unwrap()).unwrap();
            fs::write(cache_location, cache_json)?;
            Ok(())
        } else {
            println!("Not cached");
            Ok(())
        }
    }

    pub async fn run(&self, variables: Map<String, Value>, cache_location: PathBuf) -> Result<Value, QueryError> {
        println!("Running Query \"{}\" at \"{}\"", self.name, self.url);
        let ran_template = self.clone().template.run(variables.clone())?;
        match serde_json::to_value(ran_template.clone()).and_then(|x| serde_json::to_string_pretty(&x)) {
            Ok(json) => println!("{}\n", json),
            Err(e) => println!("Printing query template failed: {}", e),
        }

        let result_block = {
            match self.clone().get_cached(variables.clone(), cache_location.clone()).await {
                Ok(result) => {
                    println!("Got cached result..");
                    Ok(result)
                },
                Err(_e) => {
                    match self.query_type {
                        QueryType::Get => {
                            let client = Client::new();
                            let response = client.get(self.url.clone())
                                .json(&ran_template)
                                .send()
                                .await
                                .map_err(|e| QueryError::ReqwestError(Arc::new(e)))?;
                            if response.status().is_success() {
                                Ok(response.json().await.map_err(|e| QueryError::ReqwestError(Arc::new(e)))?)
                            } else {
                                let response_text = match response.text().await {
                                    Ok(text) => text,
                                    Err(e) => format!("error: \n{}", e),
                                };
                                Err(QueryError::RequestFailed {
                                    response: response_text,
                                })
                            }
                        },

                        QueryType::Put => {
                            let client = Client::new();
                            let response = client.put(self.url.clone())
                                .json(&ran_template)
                                .send()
                                .await
                                .map_err(|e| QueryError::ReqwestError(Arc::new(e)))?;
                            if response.status().is_success() {
                                Ok(response.json().await.map_err(|e| QueryError::ReqwestError(Arc::new(e)))?)
                            } else {
                                let response_text = match response.text().await {
                                    Ok(text) => text,
                                    Err(e) => format!("error: \n{}", e),
                                };
                                Err(QueryError::RequestFailed {
                                    response: response_text,
                                })
                            }
                        },

                    }
                },
            }
        };

        // result_block.
        match result_block {
            Ok(result) => {
                self.put_cached(result.clone(), variables, cache_location).await?;
                Ok(result)
            },
            Err(e) => Err(e),
        }
    }
}


#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Queries {
    queries: Vec<Query>,
}

impl Queries {
    pub async fn run(self, variables: Map<String, Value>, cache_location: PathBuf) -> Result<Vec<Map<String, Value>>, QueryError> {
        let mut result = Vec::with_capacity(self.queries.len());
        let mut stream = stream::iter(self.queries);

        while let Some(query) = stream.next().await {
            let query_result = query.run(variables.clone(), cache_location.clone()).await?;
            let mut query_result_json = Map::new();
            query_result_json.insert("query".to_string(), query.to_json()?);
            query_result_json.insert("result".to_string(), query_result);
            result.push(query_result_json)
        }
        Ok(result)
    }
}
//// END query


