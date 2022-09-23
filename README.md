## Zenode

In order to run this you first need to install [aquadoggo](https://github.com/p2panda/aquadoggo)

```sh
git clone https://github.com/p2panda/aquadoggo.git

RUST_LOG=aquadoggo=info cargo run
```

## Proof of concept

Inside [main.rs](./src/main.rs)

The `Operator` struct is the main wrapper around the p2panda library and the graphql layer.

To create a new `Operator` use `Operator::default()` or `Operator::new()`.
`Operator::default()` reads the ENDPOINT env variable, if unset it uses `http://localhost:2020/graphql` as default endpoint

Once `aquadoggo` is running. Run the following to test `Zenode`:

```sh
cargo test
```

- [x] Create schemas
- [x] Crate fields
- [x] Crate instance
- [x] Update instance
- [x] Delete instance
- [x] Read endpoint from env
- [x] Better field to json
- [ ] Save schema_id
- [ ] Link schema name with schema_id
- [ ] Serializable query string

## Quick start

```rs
// create an Operator
let op = Operator::default();

let mut fields = vec![("pokemon_id", "int"), ("pokemon_name", "str")];

let id = op.create_schema("POKEMON", "Pokemon schema", &mut fields).await?;

// generate schema_id
let schema_id = format!("POKEMON_{}", id);

// create an instance
let mut fields = vec![("pokemon_id", "1"), ("pokemon_name", "Bulbasaur")];
let instance_id = op.create_instance(&schema_id, &mut fields).await?;

// update instance
let mut fields = vec![("pokemon_name", "Charmander")];
let update_id = op.update_instance(&schema_id, &instance_id, &mut fields)await?;

// finally delete instance
let _delete_id = op.delete_instance(&schema_id, &update_id).await?;
```
