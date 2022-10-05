pub mod graphql;
mod operator;
mod utils;

pub use operator::*;

#[cfg(test)]
mod tests {
    use crate::{field, Operator};

    #[tokio::test]
    async fn create_schema_test() -> Result<(), String> {
        let op = Operator::default();

        // ---------
        // Test create schema

        let mut fields = vec![
            field("name", "str"),
            field("number", "int"),
            field("pi", "float"),
            field("isFree", "bool"),
        ];
        let schema_id = op.create_schema("test", "DESCRIPTION", &mut fields).await?;

        let schema_id = format!("test_{}", schema_id);

        let mut fields = vec![
            field("name", "UMBRA"),
            field("number", "69"),
            field("pi", "3.1416"),
            field("isFree", "false"),
        ];

        let instance_id = op.create_instance(&schema_id, &mut fields).await?;

        let mut fields = vec![
            field("name", "UMBRA_BEAR_420"),
            field("number", "10"),
            field("isFree", "true"),
        ];

        let update_id = op
            .update_instance(&schema_id, &instance_id, &mut fields)
            .await?;

        let _delete_id = op.delete_instance(&schema_id, &update_id).await?;

        // ---------

        // ---------
        // Test create pokemon schema

        let mut fields = vec![
            field("id", "int"),
            field("name", "str"),
            field("shiny", "bool"),
            field("exp", "float"),
        ];

        let id = op
            .create_schema("POKEMON", "Pokemon schema", &mut fields)
            .await?;

        let schema_id = format!("POKEMON_{}", id);

        // test debug
        let res = op.get_schema_definition(&id, &id).await;
        assert!(res.is_ok());

        let mut fields = vec![
            field("id", "1"),
            field("name", "Bulbasaur"),
            field("shiny", "false"),
            field("exp", "3.1416"),
        ];

        let instance_id = op.create_instance(&schema_id, &mut fields).await?;

        let mut fields = vec![field("name", "Charmander"), field("shiny", "true")];
        let update_id = op
            .update_instance(&schema_id, &instance_id, &mut fields)
            .await?;

        let _delete_id = op.delete_instance(&schema_id, &update_id).await?;

        // ---------

        Ok(())
    }

    #[tokio::test]
    async fn test_debug_fetch_schema() {
        let op = Operator::default();
        let res = op.get_all_schema_definition().await;

        assert!(res.is_ok(), "Should return all schema definitions");

        let json = serde_json::to_string_pretty(&res.unwrap()).expect("ERROR!!!");
        println!("{}", &json);
    }
}
