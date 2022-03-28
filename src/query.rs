use crate::json_template::{TValue, TValueRunError};

use std::fs;
use std::path::PathBuf;
use std::sync::{Arc};

use reqwest::{Client};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tokio_stream::{self as stream, StreamExt};
use thiserror::Error;

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
    pub fn len(&self) -> usize {
        self.queries.len()
    }

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


