use crate::fields::FieldValue;
use crate::operator::Operator;
use crate::schema::SchemaResponse;
use std::collections::HashMap;
use std::error::Error;

pub type DocumentFields = HashMap<String, FieldValue>;

pub struct DocumentFieldBuilder {
    pub map: DocumentFields,
}

impl DocumentFieldBuilder {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn field(mut self, name: &str, value: FieldValue) -> Self {
        self.map.insert(name.to_string(), value);
        self
    }

    pub fn build(self) -> DocumentFields {
        self.map
    }
}

impl Default for DocumentFieldBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct DocumentResponse<'a> {
    pub id: String,
    pub schema_id: String,
    pub fields: HashMap<String, FieldValue>,
    pub operator: &'a Operator,
}

impl<'a> DocumentResponse<'a> {
    pub fn new(
        id: &str,
        schema_id: &str,
        fields: HashMap<String, FieldValue>,
        operator: &'a Operator,
    ) -> Self {
        Self {
            id: id.to_string(),
            schema_id: schema_id.to_string(),
            fields,
            operator,
        }
    }

    pub async fn update_field(
        &self,
        name: &str,
        value: FieldValue,
    ) -> Result<String, Box<dyn Error>> {
        let fields = DocumentFieldBuilder::new().field(name, value).build();

        let view_id = self
            .operator
            .update_document(&self.schema_id, &self.id, &fields)
            .await?;

        Ok(view_id)
    }

    pub async fn update(&self, fields: DocumentFields) -> Result<String, Box<dyn Error>> {
        let view_id = self
            .operator
            .update_document(&self.schema_id, &self.id, &fields)
            .await?;
        Ok(view_id)
    }

    pub async fn delete(&self) -> Result<String, Box<dyn Error>> {
        let view_id = self
            .operator
            .delete_document(&self.schema_id, &self.id)
            .await?;
        Ok(view_id)
    }
}

pub struct DocumentBuilder<'a> {
    schema_response: &'a SchemaResponse<'a>,
    map: HashMap<String, FieldValue>,
}

impl<'a> DocumentBuilder<'a> {
    pub fn new(schema_response: &'a SchemaResponse) -> DocumentBuilder<'a> {
        Self {
            map: HashMap::new(),
            schema_response,
        }
    }

    pub fn field(mut self, key: &str, value: FieldValue) -> Self {
        self.map.insert(key.to_string(), value);
        self
    }

    pub async fn build(self) -> Result<DocumentResponse<'a>, Box<dyn Error>> {
        let document = self.schema_response.spawn(&self.map).await?;
        Ok(document)
    }
}
