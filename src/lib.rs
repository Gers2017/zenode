pub mod builder;
pub mod graphql;
mod operator;
mod utils;

pub use builder::fields::FieldType;
pub use operator::*;

#[cfg(test)]
mod tests {
    use crate::builder::SchemaBuilder;
    use crate::utils::{field_to_json, sort_fields};
    use crate::{collection_field, field, field_def, FieldType::*, Operator};

    #[tokio::test]
    async fn create_schema_test() -> Result<(), String> {
        let op = Operator::default();

        // ---------
        // Test create schema

        let id = op
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

        let schema_id = format!("test_{}", &id);

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
    async fn test_relation_list() -> Result<(), String> {
        let op = Operator::default();

        let product_schema_id = op
            .create_schema(
                "product_test",
                "Product test schema",
                &mut [field_def("name", Str)],
            )
            .await?;

        let product_schema_id = format!("product_test_{}", &product_schema_id);

        let client_schema_id = op
            .create_schema(
                "client_test",
                "Client test schema",
                &mut [field_def("products", RelationList(&product_schema_id))],
            )
            .await?;

        let client_schema_id = format!("client_test_{}", &client_schema_id);

        let product_a_id = op
            .create_instance(&product_schema_id, &mut [field("name", "product_a")])
            .await?;

        let product_b_id = op
            .create_instance(&product_schema_id, &mut [field("name", "product_b")])
            .await?;

        let _client_id = op
            .create_instance(
                &client_schema_id,
                &mut [collection_field(
                    "products",
                    &[&product_a_id, &product_b_id],
                )],
            )
            .await?;

        Ok(())
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
            field_to_json(&field("float", "40.00000")),
            String::from(r#""float": 40.0"#)
        );

        assert_eq!(
            field_to_json(&field("int", "0000099")),
            String::from(r#""int": 99"#)
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

    #[test]
    fn test_field_type() {
        assert_eq!(
            Relation("schema_02020fb20").to_string(),
            "relation(schema_02020fb20)"
        );
        assert_eq!(
            RelationList("schema_02020fb20").to_string(),
            "relation_list(schema_02020fb20)"
        );
        assert_eq!(
            PinnedRelation("schema_02020fb20").to_string(),
            "pinned_relation(schema_02020fb20)"
        );
        assert_eq!(
            PinnedRelationList("schema_02020fb20").to_string(),
            "pinned_relation_list(schema_02020fb20)"
        );
    }
}
