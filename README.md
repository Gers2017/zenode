## Zenode

## What is this project

This project is an abstraction layer between the client and the p2panda node by
providing tools to easily perform operations on a p2panda node.

### Before you start

In order to run this you first need to install [aquadoggo](https://github.com/p2panda/aquadoggo)

```sh
git clone https://github.com/p2panda/aquadoggo.git

RUST_LOG=aquadoggo=info cargo run
```

The `Operator` struct is the main wrapper around the p2panda library and the graphql layer.

To create a new `Operator` use `Operator::default()` or `Operator::new()`.

`Operator::default()` reads the `ENDPOINT` environment variable, if is not present it uses `http://localhost:2020/graphql` as default endpoint.

Run the following to test `Zenode` (aquadoggo must be running in the background):

```sh
cargo test
```

## Quick start

```rs
use zenode::{field, Operator};
use zenode::FieldType::*;

// create an Operator
let op = Operator::default();

// create a schema
let id = op.create_schema(
    "POKEMON",
    "Pokemon schema",
    &mut [
        field_def("pokemon_id", Int), // same as field("pokemon_id", "int")
        field_def("pokemon_name", Str),
    ]
).await?;

// generate schema_id
let schema_id = format!("POKEMON_{}", id);

// create an instance
let instance_id = op.create_instance(&schema_id, &mut [
    field("pokemon_id", "1"), field("pokemon_name", "Bulbasaur")
]).await?;

// update the instance
let update_id = op.update_instance(&schema_id, &instance_id, &mut [
    field("pokemon_name", "Charmander")
]).await?;

// finally delete the instance
let _delete_id = op.delete_instance(&schema_id, &update_id).await?;
```

## Experimental Schema Builder

```rs
let op = Operator::default();

let mut puppy_builder = SchemaBuilder::new("puppy", "Puppy schema", &op)
    .field("name", Str)
    .field("cuteness", Int);

puppy_builder.build().await?;

let tiramisu_id = puppy_builder
    .instantiate(&mut [field("name", "Tiramisu"), field("cuteness", "200")])
    .await?;
```

## Features

-   [x] Create schemas
-   [x] Crate fields
-   [x] Crate instance
-   [x] Update instance
-   [x] Delete instance
-   [x] Read endpoint from env
-   [x] Better field to json
-   [ ] Save schema_id
-   [ ] Link schema name with schema_id
-   [ ] Serializable query string
