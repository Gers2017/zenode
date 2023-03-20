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

    use serde::{Deserialize, Serialize};
    use tokio::time;

    async fn wait(millis: u64) {
        time::sleep(Duration::from_millis(millis)).await
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct PetSchema {
        id: u32,
        name: String,
    }

    #[tokio::test]
    async fn pet_schema_test() -> Result<(), Box<dyn Error>> {
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

        wait(100).await;

        let update_fields = DocumentFieldBuilder::new()
            .field("name", FieldValue::String("Alice".to_string()))
            .build();

        document.update(update_fields).await?;

        // test schema methods

        match pet_schema.find_many::<PetSchema>().await {
            Ok(many_pets) => {
                let first = many_pets.documents.get(0).unwrap();

                println!("{:#?}", &many_pets.documents);

                let single_pet = pet_schema
                    .find_single::<PetSchema>(&first.meta.view_id)
                    .await;

                assert!(single_pet.is_ok());
                println!("{:#?}", &single_pet.unwrap());
            }
            Err(e) => {
                panic!("Error at retrieving multiple documents: {}", e);
            }
        }

        // test get definitions

        let pet_definition = pet_schema.get_definition().await;
        assert!(pet_definition.is_ok());

        assert_eq!(pet_definition.unwrap().schema.meta.view_id, pet_schema.id);

        let all_definitions = operator.get_all_schema_definition().await;
        assert!(all_definitions.is_ok());

        Ok(())
    }
}
