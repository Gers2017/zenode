## Zenode

In order to run this you first need to install [aquadoggo](https://github.com/p2panda/aquadoggo)

```sh
git clone https://github.com/p2panda/aquadoggo.git

RUST_LOG=aquadoggo=info cargo run
```

## Proof of concept

Inside [main.rs](./src/main.rs)

The `Operator` struct is the main wrapper around the p2panda library and the graphql layer.

To create a new `Operator` use `Operator::default()` or `Operator::new()`. For the moment `Operator::default()` is configured for development (it uses `http://localhost:2020/graphql` as default endpoint)

Once `aquadoggo` is running. Run the following to test `Zenode`:

```sh
cargo run
```

Alternatively you can run:

```sh
cargo test
```

| Feature                         | Status |
| ------------------------------- | ------ |
| Create schemas                  | [x]    |
| Crate fields                    | [x]    |
| Crate instance                  | [x]    |
| Update instance                 | [x]    |
| Delete instance                 | [x]    |
| Read endpoint from env          | []     |
| Save schema_id                  | []     |
| Better field to json            | []     |
| Link schema name with schema_id | []     |
| Serializable query string       | []     |
