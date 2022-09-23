use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AllSchemaDefinitionResponse {
    pub all_schemas: Vec<SchemaDefinition>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SchemaDefinition {
    pub meta: Meta,
    pub fields: FieldsSchemaDefinition,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    pub document_id: String,
    pub view_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FieldsSchemaDefinition {
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
