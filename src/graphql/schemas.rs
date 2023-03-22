use p2panda_rs::{
    entry::{LogId, SeqNum},
    hash::Hash,
};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

// Necessary to create operations
// ------------------------------------------------

/// GraphQL response for `nextArgs` query.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NextArgsResponse {
    pub next_args: NextArguments,
}

/// GraphQL response for `publish` mutation.
// #[allow(dead_code)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct PublishResponse {
    pub publish: NextArguments,
}

/// GraphQL response giving us the next arguments to create an Bamboo entry.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NextArguments {
    pub log_id: LogId,
    pub seq_num: SeqNum,
    pub skiplink: Option<Hash>,
    pub backlink: Option<Hash>,
}

// Responses from GraphQL
// ------------------------------------------------

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AllSchemaDefinitionResponse {
    pub all_schemas: Vec<SchemaDefinition>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SchemaDefinitionResponse {
    pub schema: SchemaDefinition,
}

// GraphQL Schemas
// ------------------------------------------------

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SchemaDefinition {
    pub meta: Meta,
    pub fields: SchemaDefinitionFields,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    pub document_id: String,
    pub view_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SchemaDefinitionFields {
    pub name: String,
    pub description: String,
    pub fields: Vec<Fields>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Fields {
    pub fields: FieldDefinition,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FieldDefinition {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
}

// Var structs for GraphQL queries
// ------------------------------------------------
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetSchemaVars {
    pub view_id: String,
}
