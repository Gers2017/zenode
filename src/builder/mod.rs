pub mod fields;
use crate::builder::fields::*;
use crate::operator::*;
use std::convert::AsRef;

// ---- Builders ----

pub struct SchemaBuilder<'a> {
    pub schema_id: String,
    pub operation_id: String,
    pub name: String,
    pub description: String,
    pub fields: Vec<SchemaField<'a>>,
    pub operator: &'a Operator,
}

impl<'a> SchemaBuilder<'a> {
    pub fn new(name: &str, description: &str, operator: &'a Operator) -> SchemaBuilder<'a> {
        Self {
            schema_id: String::new(),
            operation_id: String::new(),
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

        let id = self
            .operator
            .create_schema(&self.name, &self.description, &mut fields)
            .await?;

        self.schema_id = format!("{}_{}", &self.name, &id);
        self.operation_id = id;
        Ok(())
    }

    pub async fn instantiate(&self, fields: &mut [StringTuple]) -> Result<String, String> {
        self.operator.create_instance(&self.schema_id, fields).await
    }

    pub fn factory(&self) -> InstanceFactory {
        InstanceFactory::new(
            &self.schema_id,
            &self.operation_id,
            self.operator,
            &self.fields,
        )
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

use std::collections::HashMap;

pub struct InstanceFactory<'a> {
    pub schema_id: String,
    pub operation_id: String,
    pub registered_fields: HashMap<String, FieldType<'a>>,
    pub operator: &'a Operator,
}

impl<'a> InstanceFactory<'a> {
    fn new(
        schema_id: &str,
        operation_id: &str,
        operator: &'a Operator,
        field_definitions: &[SchemaField<'a>],
    ) -> InstanceFactory<'a> {
        let mut registered_fields: HashMap<String, FieldType> = HashMap::new();

        field_definitions.iter().for_each(|v| {
            let SchemaField { name, field_type } = v;

            if !registered_fields.contains_key(name) {
                registered_fields.insert(name.to_string(), field_type.clone());
            } else {
                todo!("Add a warning about duplicated keys");
            }
        });

        Self {
            operator: operator,
            schema_id: schema_id.to_string(),
            operation_id: operation_id.to_string(),
            registered_fields: registered_fields,
        }
    }

    fn verify_fields(&self, fields: &'a [StringTuple]) -> Option<&'a str> {
        for (key, _) in fields.iter() {
            if !self.registered_fields.contains_key(key) {
                return Some(key.as_str());
            }
        }

        None
    }

    pub async fn create(&self, fields: &mut [StringTuple]) -> Result<String, String> {
        let invalid_key = self.verify_fields(fields);
        if let Some(key) = invalid_key {
            return Err(format!(
                "Unknown key '{key}' found. Schema doesn't have '{key}' key"
            ));
        }

        self.operator.create_instance(&self.schema_id, fields).await
    }

    pub async fn update(
        &self,
        view_id: &str,
        update_fields: &mut [StringTuple],
    ) -> Result<String, String> {
        self.operator
            .update_instance(&self.schema_id, view_id, update_fields)
            .await
    }

    pub async fn delete(&self, view_id: &str) -> Result<String, String> {
        self.operator
            .delete_instance(&self.schema_id, view_id)
            .await
    }
}

#[tokio::test]
async fn test_instance_factory() -> Result<(), String> {
    let op = Operator::default();
    let mut s1 = SchemaBuilder::new("SCHEMA_1", "TEST SCHEMA", &op)
        .field("A", FieldType::Int)
        .field("B", FieldType::Bool);

    s1.build().await?;

    let instance_fac = s1.factory();
    let _instance_id = instance_fac
        .create(&mut [field("A", "100"), field("B", "false")])
        .await?;

    Ok(())
}
