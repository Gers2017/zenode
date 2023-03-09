// pub mod builder;
pub mod document;
pub mod fields;
pub mod graphql;
pub mod operator;
pub mod schema;
mod utils;

pub use fields::*;
pub use operator::*;

#[cfg(test)]
mod tests {
    use crate::{
        document::{DocumentBuilder, DocumentFieldBuilder},
        schema::SchemaBuilder,
        FieldType, FieldValue, Operator,
    };
    use std::{error::Error, time::Duration};

    use tokio::time;

    async fn wait(millis: u64) {
        time::sleep(Duration::from_millis(millis)).await
    }

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

        wait(1000).await;

        let update_fields = DocumentFieldBuilder::new()
            .field("name", FieldValue::String("Nekopara Fan!".to_string()))
            .build();

        document.update(update_fields).await?;

        Ok(())
    }
}
