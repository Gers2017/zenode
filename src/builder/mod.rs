pub mod fields;
use crate::builder::fields::*;
use crate::operator::*;
use std::convert::AsRef;

// ---- Builders ----

pub struct SchemaBuilder<'a> {
    pub schema_id: String,
    pub name: String,
    pub description: String,
    pub fields: Vec<SchemaField<'a>>,
    pub operator: &'a Operator,
}

impl<'a> SchemaBuilder<'a> {
    pub fn new(name: &str, description: &str, operator: &'a Operator) -> SchemaBuilder<'a> {
        Self {
            schema_id: String::new(),
            name: name.to_string(),
            description: description.to_string(),
            fields: Vec::new(),
            operator,
        }
    }

    pub fn field(mut self, field_name: &str, field_type: FieldType<'a>) -> Self {
        self.fields
            .push(SchemaField::new(field_name.to_string(), field_type));
        self
    }

    pub async fn build(&mut self) -> Result<(), String> {
        // struct schema field -> (name, type)
        let mut fields: Vec<StringTuple> = self
            .fields
            .iter()
            .map(|f| -> StringTuple { (f.name.clone(), f.field_type.to_string()) })
            .collect();

        let schema_id = self
            .operator
            .create_schema(&self.name, &self.description, &mut fields)
            .await?;

        self.schema_id = schema_id;
        Ok(())
    }

    pub async fn instantiate(&self, fields: &mut [StringTuple]) -> Result<String, String> {
        self.operator.create_instance(&self.schema_id, fields).await
    }
}

impl<'a> AsRef<SchemaBuilder<'a>> for SchemaBuilder<'a> {
    fn as_ref(&self) -> &SchemaBuilder<'a> {
        self
    }
}

pub struct SchemaField<'a> {
    pub name: String,
    pub field_type: FieldType<'a>,
}

impl SchemaField<'_> {
    pub fn new(name: String, field_type: FieldType<'_>) -> SchemaField {
        SchemaField { name, field_type }
    }
}
