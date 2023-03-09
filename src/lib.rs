// pub mod builder;
pub mod graphql;
mod operator;
mod utils;

pub use operator::*;

#[cfg(test)]
mod tests {
    // use crate::builder::SchemaBuilder;
    // use crate::utils::{field_to_json, sort_fields};
    // use crate::{Operator};

    use std::{error::Error, time::Duration};

    use crate::{
        DocumentBuilder, DocumentFieldBuilder, FieldType, FieldValue, Operator, SchemaBuilder,
    };

    #[tokio::test]
    async fn shironeko() -> Result<(), Box<dyn Error>> {
        let operator = Operator::default();

        let pet_schema = SchemaBuilder::new(&operator, "PetSchema", "description")
            .field("id", FieldType::Int)
            .field("name", FieldType::String)
            .build()
            .await?;

        let document = DocumentBuilder::new(&pet_schema)
            .field("id", FieldValue::Int(1200))
            .field("name", FieldValue::String("Neko".to_string()))
            .build()
            .await?;

        tokio::time::sleep(Duration::from_secs(5)).await;

        let update_fields = DocumentFieldBuilder::new()
            .field("name", FieldValue::String("Nekopara Fan!".to_string()))
            .build();

        document.update(update_fields).await?;

        Ok(())
    }
}
