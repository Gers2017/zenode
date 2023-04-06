use crate::document::{DocumentFields, DocumentResponse};
use crate::fields::FieldType;
use crate::graphql::schemas::SchemaDefinitionResponse;
use crate::operator::Operator;
use gql_client::GraphQLError;
use std::collections::HashMap;
use std::error::Error;

pub type SchemaFields = HashMap<String, FieldType>;

pub struct SchemaFieldBuilder {
    pub map: SchemaFields,
}

impl SchemaFieldBuilder {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn field(mut self, name: &str, value: FieldType) -> Self {
        self.map.insert(name.to_string(), value);
        self
    }

    pub fn build(self) -> SchemaFields {
        self.map
    }
}

impl Default for SchemaFieldBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SchemaResponse<'a> {
    pub id: String,
    pub name: String,
    pub fields: SchemaFields,
    pub operator: &'a Operator,
}

impl<'a> SchemaResponse<'a> {
    pub fn new(id: &str, name: &str, fields: SchemaFields, operator: &'a Operator) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            fields,
            operator,
        }
    }

    pub async fn spawn(&self, fields: &DocumentFields) -> Result<DocumentResponse, Box<dyn Error>> {
        let document = self
            .operator
            .create_document(&self.get_schema_id(), fields)
            .await?;
        Ok(document)
    }

    pub fn get_schema_id(&self) -> String {
        format!("{}_{}", self.name, self.id)
    }

    pub async fn get_definition(&self) -> Result<SchemaDefinitionResponse, GraphQLError> {
        self.operator.get_schema_definition(&self.id).await
    }
}

pub struct SchemaBuilder<'a> {
    operator: &'a Operator,
    name: String,
    description: String,
    map: SchemaFields,
}

impl<'a> SchemaBuilder<'a> {
    pub fn new(operator: &'a Operator, name: &str, description: &str) -> SchemaBuilder<'a> {
        SchemaBuilder {
            name: name.to_string(),
            description: description.to_string(),
            map: HashMap::new(),
            operator,
        }
    }

    pub fn field(mut self, name: &str, value: FieldType) -> Self {
        self.map.insert(name.to_string(), value);
        self
    }

    pub async fn build(self) -> Result<SchemaResponse<'a>, Box<dyn Error>> {
        let schema = self
            .operator
            .create_schema(&self.name, &self.description, &self.map)
            .await?;
        Ok(schema)
    }
}
