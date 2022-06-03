use crate::json_template::{TValue, TValueRunError};

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use thiserror::Error;
use tokio_stream::{self as stream, StreamExt};

/// HTTP request type
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryType {
    /// GET request
    Get,
    /// PUT request
    Put,
}

/// A Query template, see Query for additional fields required to run it.
/// This struct is deserialized from an input file by the CLI.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryTemplate {
    /// Query name, used for caching, display, and is exposed in the result
    pub name: String,

    /// Query URL
    pub url: String,

    /// Query JSON template
    pub template: TValue,

    /// Whether the result should be cached
    pub cached: bool,

    /// HTTP request type
    pub query_type: QueryType,
}

/// Error encountered when running a Query
#[derive(Clone, Debug, Error)]
pub enum QueryError {
    /// The value is not cached
    #[error("Query::get_cached: value not cached:\n{name:?}\n{url:?}")]
    NotCached {
        /// Query name
        name: String,
        /// Request URL
        url: String,
    },

    /// Running the reqwest request failed
    #[error("Query::run: request failed:\nresponse:\n{response}")]
    RequestFailed {
        /// Response pretty-printed JSON
        response: String,
    },

    /// Error when running query TValue
    #[error("TValueRunError:\n{0:?}")]
    TValueRunError(TValueRunError),

    /// reqwest::Error
    #[error("ReqwestError:\n{0}")]
    ReqwestError(Arc<reqwest::Error>),

    /// std::io::Error
    #[error("StdIoError:\n{0}")]
    StdIoError(Arc<std::io::Error>),

    /// serde_json::Error
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

impl QueryTemplate {
    /// Convert to a Value
    pub fn to_json(&self) -> Result<Value, QueryError> {
        Ok(serde_json::to_value(self)?)
    }

    /// Convert to a Query with the given variables, cache_location, resp.
    pub fn to_query(self, variables: Arc<Map<String, Value>>, cache_location: Arc<PathBuf>) -> Query {
        Query {
            query_template: self,
            variables,
            cache_location,
        }
    }
}

/// QueryTemplate with variables to instantiate it with and a cache location
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Query {
    query_template: QueryTemplate,
    variables: Arc<Map<String, Value>>,
    cache_location: Arc<PathBuf>,
}

impl Query {
    /// Index string to cache this Query
    pub fn cache_index(&self) -> String {
        format!("{:?}:{:?}", self.query_template.name, self.variables)
    }

    /// Get the Query::cache_index value at the given cache_location
    pub async fn get_cached(&self) -> Result<Value, QueryError> {
        if self.query_template.cached {
            println!("Checking cache: {:?}", self.cache_location.clone());
            let cache_str = fs::read_to_string((*self.cache_location).clone())?;
            let cache: Map<String, Value> = serde_json::from_str(&cache_str)?;
            cache.get(&self.cache_index()).ok_or_else(|| {
                QueryError::NotCached {
                    name: self.query_template.name.clone(),
                    url: self.query_template.url.clone(),
            }}).map(|x| x.clone())
        } else {
            Err(QueryError::NotCached {
                name: self.query_template.name.clone(),
                url: self.query_template.url.clone(),
            })
        }
    }

    /// Put the given result Value in the given cache_location at Query::cache_index,
    /// overwriting any existing cached result
    pub async fn put_cached(&self, result: Value) -> Result<(), QueryError> {
        if self.query_template.cached {
            println!("Adding to cache: {:?}", self.cache_location.clone());
            let mut cache: Map<String, Value> = if self.cache_location.as_path().exists() {
                let cache_str = fs::read_to_string((*self.cache_location).clone())?;
                serde_json::from_str(&cache_str)?
            } else {
                Map::new()
            };
            cache.insert(self.cache_index(), result);
            let cache_json = serde_json::to_string_pretty(&serde_json::to_value(cache).unwrap()).unwrap();
            fs::write((*self.cache_location).clone(), cache_json)?;
            Ok(())
        } else {
            println!("Not cached");
            Ok(())
        }
    }

    /// Run queries by:
    /// 1. Instantiating the template with the given variables
    /// 2. Converting the template to JSON
    /// 3. Looking up the query in the cache
    /// 4. If not found, dispatch along QueryType, sending using reqwest
    /// 5. Cache response if successful
    pub async fn run(&self) -> Result<Value, QueryError> {
        println!("Running Query \"{}\" at \"{}\"", self.query_template.name, self.query_template.url);
        let ran_template = self.clone().query_template.template.run((*self.variables).clone())?;
        match serde_json::to_value(ran_template.clone()).and_then(|x| serde_json::to_string_pretty(&x)) {
            Ok(json) => println!("{}\n", json),
            Err(e) => println!("Printing query template failed: {}", e),
        }
        match self.clone().get_cached().await {
            Ok(result) => {
                println!("Got cached result..\n");
                Ok(result)
            },
            Err(_e) => {
                let client = Client::new();
                let request_builder = match self.query_template.query_type {
                    QueryType::Get => {
                        client.get(self.query_template.url.clone())
                    },
                    QueryType::Put => {
                        client.put(self.query_template.url.clone())
                    },
                };
                let response = request_builder
                    .json(&ran_template)
                    .send()
                    .await
                    .map_err(|e| QueryError::ReqwestError(Arc::new(e)))?;
                if response.status().is_success() {
                    let result: Value = response.json()
                        .await
                        .map_err(|e| QueryError::ReqwestError(Arc::new(e)))?;
                    self.put_cached(result.clone()).await?;
                    Ok(result)
                } else {
                    let response_text = response.text()
                        .await
                        .unwrap_or_else(|e| format!("error: \n{}", e));
                    Err(QueryError::RequestFailed {
                        response: response_text,
                    })
                }
            },
        }
    }
}

/// An ordered series of QueryTemplates
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryTemplates {
    queries: Vec<QueryTemplate>,
}

impl QueryTemplates {
    /// Number of queries
    pub fn len(&self) -> usize {
        self.queries.len()
    }

    /// Is number of queries empty?
    pub fn is_empty(&self) -> bool {
        self.queries.is_empty()
    }

    /// Run a list of QueryTemplate's, in series, and collect their results
    pub async fn run(self, variables: Arc<Map<String, Value>>, cache_location: Arc<PathBuf>) -> Result<Vec<Map<String, Value>>, QueryError> {
        let mut result = Vec::with_capacity(self.queries.len());
        let mut stream = stream::iter(self.queries);
        while let Some(query_template) = stream.next().await {
            let query_json = query_template.to_json()?;
            let query_result = query_template.to_query(variables.clone(), cache_location.clone()).run().await?;
            let mut query_result_json = Map::new();
            query_result_json.insert("query".to_string(), query_json);
            query_result_json.insert("result".to_string(), query_result);
            result.push(query_result_json)
        }
        Ok(result)
    }
}
