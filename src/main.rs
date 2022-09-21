use zenode::Operator;

#[tokio::main]
async fn main() {
    let operator = Operator::default();

    operator.print_public_key_debug();

    let mut fields = vec![("pokemon_id", "int"), ("pokemon_name", "str")];
    let id = operator
        .create_schema("POKEMON", "Pokemon schema", &mut fields)
        .await;

    let schema_id = format!("POKEMON_{}", id);

    let mut fields = vec![("pokemon_id", "1"), ("pokemon_name", "Bulbasaur")];
    let instance_id = operator.create_instance(&schema_id, &mut fields).await;

    let update_id = operator
        .update_instance(
            &schema_id,
            &instance_id,
            &mut vec![("pokemon_name", "Charmander")],
        )
        .await;

    operator.delete_instance(&schema_id, &update_id).await;
}
