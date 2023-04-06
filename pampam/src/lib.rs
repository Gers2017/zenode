use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::{error::Error, fmt::Display};

pub mod convert;
pub mod parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct PamPamCli {
    /// Endpoint for p2panda node. Default is `http://localhost:2020/graphql`
    #[clap(short, long)]
    pub endpoint: Option<String>,

    // Path to the key pair
    #[clap(short, long)]
    pub key_pair_path: Option<PathBuf>,

    #[clap(short, long)]
    pub op_version: Option<usize>,

    #[clap(subcommand)]
    pub commands: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[clap(subcommand)]
    Create(CreateCommands),

    #[clap(subcommand)]
    Update(UpdateCommands),

    #[clap(subcommand)]
    Delete(DeleteCommands),
}

#[derive(Subcommand, Debug)]
pub enum CreateCommands {
    Schema {
        name: String,
        description: String,
        fields: Vec<String>,
    },
    Document {
        schema_id: String,
        fields: Vec<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum UpdateCommands {
    Document {
        schema_id: String,
        view_id: String,
        fields: Vec<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum DeleteCommands {
    Document { schema_id: String, view_id: String },
}

#[derive(Debug)]
pub enum PamError {
    ParseError { field: String, reason: String },
    UnknownTypeError { typ: String },
    GenericStringError(String),
}

impl Display for PamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use PamError::*;
        match self {
            ParseError { field, reason } => write!(f, "Error at parsing {}, {}", field, reason),
            UnknownTypeError { typ } => write!(f, "Unknown type \"{}\"", typ),
            GenericStringError(s) => write!(f, "{}", s),
        }
    }
}

impl Error for PamError {}

impl From<String> for PamError {
    fn from(value: String) -> Self {
        PamError::GenericStringError(value)
    }
}
