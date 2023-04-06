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

    /// Path to the key pair
    #[clap(short, long)]
    pub key_pair_path: Option<PathBuf>,

    #[clap(short, long)]
    pub op_version: Option<usize>,

    #[clap(subcommand)]
    pub commands: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Subcommand to create schemas or documents
    #[clap(subcommand)]
    Create(CreateCommands),

    /// Subcommand to update a document
    #[clap(subcommand)]
    Update(UpdateCommands),

    /// Subcommand to delete a document
    #[clap(subcommand)]
    Delete(DeleteCommands),
}

#[derive(Subcommand, Debug)]
pub enum CreateCommands {
    /// Returns the `schema_id` (schema_name_0000123) of the new schema
    Schema {
        /// Name of the schema
        name: String,
        /// Description for the schema
        description: String,
        /// Schema fields in the shape `foo: int` or `bar: relation(schema_name_0000123)`.
        fields: Vec<String>,
    },
    /// Returns the `document_id` (0020735fce52d) of the new document
    Document {
        /// Schema id to create the document, example: `schema_name_0000123`
        schema_id: String,
        /// Document fields in the shape `foo: 100` or `bar: relation(0000123)`
        fields: Vec<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum UpdateCommands {
    /// Returns the `document_id` (0020735fce52d) if the updated document
    Document {
        /// The schema id of the document to update, example: `schema_name_0000123`
        schema_id: String,
        /// The view id of the document to update, example: `0020735fce52d`
        view_id: String,
        /// Document fields in the shape `foo: 100` or `bar: relation(0000123)`
        fields: Vec<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum DeleteCommands {
    /// Returns the `document_id` (0020735fce52d) if the delete document
    Document {
        /// The schema id of the document to update, example: `schema_name_0000123`
        schema_id: String,
        /// The view id of the document to delete, example: `0020735fce52d`
        view_id: String,
    },
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
