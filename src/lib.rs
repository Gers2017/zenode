pub mod builder;
pub mod graphql;
mod operator;
mod utils;

pub use operator::*;

#[cfg(test)]
mod tests {
    use crate::builder::fields::FieldType::*;
    use crate::builder::SchemaBuilder;
    use crate::utils::{field_to_json, sort_fields};
    use crate::{field, field_def, Operator};

    #[tokio::test]
    async fn create_schema_test() -> Result<(), String> {
        let op = Operator::default();

        // ---------
        // Test create schema

        let schema_id = op
            .create_schema(
                "test",
                "DESCRIPTION",
                &mut [
                    field_def("name", Str),
                    field_def("number", Int),
                    field_def("pi", Float),
                    field_def("isFree", Bool),
                ],
            )
            .await?;

        let schema_id = format!("test_{}", schema_id);

        let instance_id = op
            .create_instance(
                &schema_id,
                &mut [
                    field("name", "UMBRA"),
                    field("number", "69"),
                    field("pi", "3.1416"),
                    field("isFree", "false"),
                ],
            )
            .await?;

        let update_id = op
            .update_instance(
                &schema_id,
                &instance_id,
                &mut [
                    field("name", "UMBRA_BEAR"),
                    field("number", "10"),
                    field("pi", "4.0"),
                    field("isFree", "true"),
                ],
            )
            .await?;

        let _delete_id = op.delete_instance(&schema_id, &update_id).await?;

        // test get_schema_definition
        let id = schema_id.replace("test_", "");
        let res = op.get_schema_definition(&id, &id).await;
        assert!(res.is_ok());

        // ---------

        Ok(())
    }

    #[tokio::test]
    async fn test_debug_fetch_schema() {
        let op = Operator::default();
        let res = op.get_all_schema_definition().await;
        assert!(res.is_ok(), "Should return all schema definitions");

        let json = serde_json::to_string_pretty(&res.unwrap());
        assert!(json.is_ok());
    }

    #[tokio::test]
    async fn test_schema_builder() -> Result<(), String> {
        let op = Operator::default();

        let mut parent_builder = SchemaBuilder::new("parent", "PARENT TEST SCHEMA", &op)
            .field("name", Str)
            .field("points", Int);

        parent_builder.build().await?;

        let parent_schema_id = parent_builder.schema_id.as_str();

        let mut pet_builder = SchemaBuilder::new("pet", "PET TEST SCHEMA", &op)
            .field("name", Str)
            .field("parent", Relation(parent_schema_id));

        pet_builder.build().await?;

        let parent_instance_id = parent_builder
            .instantiate(&mut [field("name", "Alice"), field("points", "100")])
            .await?;

        pet_builder
            .instantiate(&mut [field("name", "Blue"), field("parent", &parent_instance_id)])
            .await?;

        Ok(())
    }

    #[test]
    fn test_field_to_json() {
        assert_eq!(
            field_to_json(&field("name", "false")),
            String::from(r#""name": false"#)
        );

        assert_eq!(
            field_to_json(&field("name", "Bob")),
            String::from(r#""name": "Bob""#)
        );

        assert_eq!(
            field_to_json(&field("number", "1000")),
            String::from(r#""number": 1000"#)
        );

        assert_eq!(
            field_to_json(&field("float", "387.927")),
            String::from(r#""float": 387.927"#)
        );

        assert_eq!(
            field_to_json(&field("vec", r#"["id_020208973fb0"]"#)),
            String::from(r#""vec": ["id_020208973fb0"]"#)
        );
    }

    #[test]
    fn test_sort_fields() {
        let fields = &mut [
            field("omega", "_"),
            field("delta", "_"),
            field("gamma", "_"),
            field("alpha", "_"),
            field("beta", "_"),
        ];

        sort_fields(fields);

        let expected = &[
            field("alpha", "_"),
            field("beta", "_"),
            field("delta", "_"),
            field("gamma", "_"),
            field("omega", "_"),
        ];

        for (i, f) in fields.iter().enumerate() {
            let (expected_name, _) = &expected[i];
            let (name, _) = f;
            assert_eq!(*name, *expected_name);
        }
    }
}
