// pub mod builder;
pub mod graphql;
mod operator;
mod utils;

// pub use builder::fields::FieldType;
pub use operator::*;

#[cfg(test)]
mod tests {
    // use crate::builder::SchemaBuilder;
    // use crate::utils::{field_to_json, sort_fields};
    // use crate::{Operator};

    use std::error::Error;

    use crate::{DocumentBuilder, FieldType, FieldValue, Operator, SchemaBuilder};

    #[tokio::test]
    async fn shironeko() -> Result<(), Box<dyn Error>> {
        let operator = Operator::default();

        let pet_schema = SchemaBuilder::new(&operator, "PetSchema", "description")
            .field("id", FieldType::Number)
            .field("name", FieldType::String)
            .build()
            .await?;

        let document = DocumentBuilder::new(&pet_schema)
            .field("id", FieldValue::Number(120))
            .field("name", FieldValue::String("Neko".to_string()))
            .build()
            .await?;

        Ok(())
    }
}
