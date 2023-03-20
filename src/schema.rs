use crate::document::{DocumentFields, DocumentResponse};
use crate::fields::FieldType;
use crate::graphql::schemas::{ManyDocumentsResponse, SingleDocumentResponse};
use crate::operator::Operator;
use gql_client::GraphQLError;
use serde::de::DeserializeOwned;
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

pub struct SchemaResponse<'a> {
    pub id: String,
    pub name: String,
    pub fields: HashMap<String, FieldType>,
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

    pub async fn find_single<T>(
        &self,
        view_id: &str,
    ) -> Result<SingleDocumentResponse<T>, GraphQLError>
    where
        T: DeserializeOwned,
    {
        let field_query = self.field_keys().join(",");
        let query = format!(
            r#"query single_document {{  document: {}(viewId: "{}") {{ meta {{ documentId, viewId }} fields {{ {} }} }}  }}"#,
            self.get_schema_id(),
            view_id,
            field_query
        );

        self.operator
            .client
            .query_unwrap::<SingleDocumentResponse<T>>(&query)
            .await
    }

    pub async fn find_many<T>(&self) -> Result<ManyDocumentsResponse<T>, GraphQLError>
    where
        T: DeserializeOwned,
    {
        let field_query = self.field_keys().join(", ");

        let query = format!(
            r#"query many_document {{  documents: all_{} {{ meta {{ viewId documentId }} fields {{ {} }} }}  }}"#,
            self.get_schema_id(),
            field_query
        );

        self.operator
            .client
            .query_unwrap::<ManyDocumentsResponse<T>>(&query)
            .await
    }

    pub fn field_keys(&self) -> Vec<String> {
        self.fields.keys().cloned().collect()
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
