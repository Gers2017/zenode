use crate::document::{DocumentFields, DocumentResponse};
use crate::fields::FieldType;
use crate::operator::Operator;
use std::collections::HashMap;
use std::error::Error;

pub type SchemaFields = HashMap<String, FieldType>;

pub struct SchemaResponse<'a> {
    pub id: String,
    pub name: String,
    pub fields: HashMap<String, FieldType>,
    pub operator: &'a Operator,
}

impl SchemaResponse<'_> {
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

    pub fn find_by_id(&self, view_id: &str) -> DocumentResponse {
        todo!("Not implemented yet");
    }
    pub fn find_many(&self, take: usize, skip: usize) -> Vec<DocumentResponse> {
        todo!("Not implemented yet");
    }
}

pub struct SchemaBuilder<'a> {
    operator: &'a Operator,
    name: String,
    description: String,
    map: HashMap<String, FieldType>,
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
