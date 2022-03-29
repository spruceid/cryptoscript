use crate::elem_type::{StackType};
use crate::stack::{Stack};
use crate::types_scratch::{ElemsPopError};
use crate::untyped_instruction::{InstructionError};
use crate::typed_instruction::{StackInstructionError};
use crate::typed_instrs::{Instrs};
use crate::parse::{parse_json, ParseError};
use crate::query::{QueryError, Queries};

use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use serde_json::Value;
use thiserror::Error;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// Queries to run
    #[clap(short, long, parse(from_os_str), value_name = "FILE")]
    queries: PathBuf,

    /// Query cache json file
    #[clap(long, parse(from_os_str), value_name = "FILE")]
    cache_location: PathBuf,

    /// Query variables (in JSON)
    #[clap(short, long)]
    variables: String,

    /// Cryptoscript program to run
    #[clap(short, long, parse(from_os_str), value_name = "FILE")]
    code: PathBuf,

    /// JSON input
    #[clap(short, long, parse(from_os_str), value_name = "FILE")]
    input: Option<PathBuf>,

    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse only
    Parse,

    /// Type check only (monomorphic)
    TypeMono,

    // // TODO: implement /// Type check only (polymorphic)
    // Type,
}

#[derive(Clone, Debug, Error)]
pub enum CliError {
    #[error("Cli::get_input: invalid input path:\n{input_path:?}")]
    InvalidInputPath {
        input_path: Option<PathBuf>,
    },

    #[error("ElemsPopError:\n{0}")]
    ElemsPopError(ElemsPopError),

    #[error("QueryError:\n{0}")]
    QueryError(QueryError),

    #[error("StackInstructionError:\n{0}")]
    StackInstructionError(StackInstructionError),

    #[error("InstructionError:\n{0}")]
    InstructionError(InstructionError),

    #[error("ParseError:\n{0}")]
    ParseError(Arc<ParseError>),

    #[error("std::io::Error:\n{0}")]
    IOError(Arc<io::Error>),

    #[error("Cli::get_input: serde_json::from_str threw error:\n{0}")]
    SerdeJsonError(Arc<serde_json::Error>),
}

impl From<ElemsPopError> for CliError {
    fn from(error: ElemsPopError) -> Self {
        Self::ElemsPopError(error)
    }
}

impl From<QueryError> for CliError {
    fn from(error: QueryError) -> Self {
        Self::QueryError(error)
    }
}

impl From<StackInstructionError> for CliError {
    fn from(error: StackInstructionError) -> Self {
        Self::StackInstructionError(error)
    }
}

impl From<InstructionError> for CliError {
    fn from(error: InstructionError) -> Self {
        Self::InstructionError(error)
    }
}

impl From<ParseError> for CliError {
    fn from(error: ParseError) -> Self {
        Self::ParseError(Arc::new(error))
    }
}
impl From<io::Error> for CliError {
    fn from(error: io::Error) -> Self {
        Self::IOError(Arc::new(error))
    }
}

impl From<serde_json::Error> for CliError {
    fn from(error: serde_json::Error) -> Self {
        Self::SerdeJsonError(Arc::new(error))
    }
}

impl Cli {
    pub fn parse_queries(&self) -> Result<Queries, CliError> {
        let queries_str = fs::read_to_string(self.queries.clone())?;
        let queries: Queries = serde_json::from_str(&queries_str)?;
        Ok(queries)
    }

    pub fn parse_code(&self) -> Result<Instrs, CliError> {
        let instructions_str = fs::read_to_string(self.code.clone())?;
        Ok(parse_json(&instructions_str)?.to_instrs()?)
    }

    pub fn get_input(&self) -> Result<Value, CliError> {
        if let Some(input_path) = self.input.as_deref() {
            let input_str = fs::read_to_string(input_path)?;
            Ok(serde_json::from_str(&input_str)?)
        } else {
            Err(CliError::InvalidInputPath {
                input_path: self.input.clone(),
            })
        }
    }

    pub fn type_of_mono(&self) -> Result<StackType, CliError> {
        let instructions = self.parse_code()?;
        let num_queries = self.parse_queries()?.len();
        instructions.debug()?;
        Ok(instructions.type_of_mono(num_queries)?)
    }

    pub async fn parse_and_run_result(&self) -> Result<(), CliError> {
        let instructions = self.parse_code()?;
        let mut stack = Stack::new();

        let input_json_value = self.get_input()?;
        stack.push_elem(input_json_value);

        let variables = serde_json::from_str(&self.variables)?;
        let mut queries_result = self.parse_queries()?.run(variables, self.cache_location.clone()).await?;
        queries_result.reverse();
        for query_result in queries_result {
            stack.push_elem(query_result)
        }
        Ok(instructions.run(&mut stack)?)
    }

    pub async fn parse_and_run(&self) -> () {
        match self.parse_and_run_result().await {
            Ok(()) => println!("successful!"),
            Err(e) => println!("failed:\n{}\n", e),
        }
    }

    pub async fn run(&self) -> () {
        match self.command {
            None => self.parse_and_run().await,
            Some(Commands::Parse) => {
                match self.parse_code() {
                    Ok(parsed) => {
                        parsed.debug()
                            .unwrap_or_else(|e| println!("Instrs::debug() failed:\n{}", e))
                    },
                    Err(e) => println!("parsing failed:\n{}", e),
                }
            },
            Some(Commands::TypeMono) => {
                match self.type_of_mono() {
                    Ok(type_of) => println!("type:\n{}", type_of),
                    Err(e) => println!("type-mono failed:\n{}", e),
                }
            },
        }
    }
}

