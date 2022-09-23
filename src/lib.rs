mod queries;

use dotenv::dotenv;
use gql_client::Client;
use p2panda_rs::{
    self,
    entry::{encode::sign_and_encode_entry, traits::AsEncodedEntry},
    entry::{LogId, SeqNum},
    hash::Hash,
    identity::KeyPair,
    operation::{encode::encode_plain_operation, plain::PlainOperation, traits::Actionable},
};
use queries::AllSchemaDefinitionResponse;
use serde::Deserialize;
use std::fmt::{Debug, Display};
use std::fs::{read_to_string, File};
use std::io::Write;
use std::path::{Path, PathBuf};

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
enum OperationAction {
    CREATE = 0,
    UPDATE = 1,
    DELETE = 2,
}

impl Display for OperationAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", *self as usize)
    }
}

type StrTuple<'a> = (&'a str, &'a str);

/// GraphQL response for `nextArgs` query.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct NextArgsResponse {
    next_args: NextArguments,
}

/// GraphQL response for `publish` mutation.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct PublishResponse {
    publish: NextArguments,
}

/// GraphQL response giving us the next arguments to create an Bamboo entry.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct NextArguments {
    log_id: LogId,
    seq_num: SeqNum,
    skiplink: Option<Hash>,
    backlink: Option<Hash>,
}

const DEFAULT_ENDPOINT: &str = "http://localhost:2020/graphql";

pub struct Operator {
    version: usize,
    key_pair: KeyPair,
    client: Client,
}

impl Operator {
    /// Creates a new Operator from scratch
    pub fn new(version: usize, path: Option<PathBuf>, endpoint: &str) -> Self {
        Operator {
            version,
            key_pair: get_key_pair(path),
            client: Client::new(endpoint),
        }
    }

    /// Creates a new Operator with default values
    /// `version: 1, path: "key.txt", endpoint: "http://localhost:2020/graphql"`
    pub fn default() -> Self {
        dotenv().ok();
        let endpoint = std::env::var("ENDPOINT").unwrap_or(DEFAULT_ENDPOINT.to_string());
        Operator::new(1, None, &endpoint)
    }

    pub async fn create_schema<'a>(
        &self,
        name: &str,
        description: &str,
        fields: &mut Vec<StrTuple<'a>>,
    ) -> Result<String, String> {
        // publish fields to node and retrieve field_ids
        let field_ids = self.publish_fields(fields).await?;

        // create schema with field_ids
        self.publish_schema(name, description, &field_ids).await
    }

    async fn publish_schema(
        &self,
        name: &str,
        description: &str,
        field_ids: &Vec<String>,
    ) -> Result<String, String> {
        let field_content: String = field_ids
            .iter()
            .map(|it| format!("[\"{}\"]", it))
            .collect::<Vec<_>>()
            .join(", ");
        /*
            Fields should have the following shape:
            "fields": [
              ["<field_id>"],
              ["<field_id>"]
            ],
        */
        let json = format!(
            r#"[{}, {}, "schema_definition_v1", {{ "description": "{}", "fields": [{}], "name": "{}" }}]"#,
            self.version,
            OperationAction::CREATE,
            description,
            field_content,
            name
        );

        self.send_to_node(&json).await
    }

    async fn publish_fields<'a>(
        &self,
        fields: &mut Vec<StrTuple<'a>>,
    ) -> Result<Vec<String>, String> {
        sort_fields(fields);

        let mut field_ids: Vec<String> = Vec::with_capacity(fields.len());

        for (name, f_type) in fields.iter() {
            let json = format!(
                r#"[{}, {}, "schema_field_definition_v1", {{ "name": "{}", "type": "{}" }}]"#,
                self.version,
                OperationAction::CREATE,
                name,
                f_type
            );

            let id = self.send_to_node(&json).await?;
            field_ids.push(id);
        }

        Ok(field_ids)
    }

    pub async fn create_instance<'a>(
        &self,
        schema_id: &str,
        fields: &mut Vec<StrTuple<'a>>,
    ) -> Result<String, String> {
        // (str, str)[] -> '"str": "str"'[]
        sort_fields(fields);
        let payload_content: Vec<String> = fields_to_json_fields(fields);

        // [1, 0, "chat_0020cae3b...", {"msg": "...", "username": "..." } ]

        let json = format!(
            r#"[{}, {}, "{}", {{ {} }} ]"#,
            self.version,
            OperationAction::CREATE,
            schema_id,
            payload_content.join(", ")
        );

        self.send_to_node(&json).await
    }

    pub async fn update_instance<'a>(
        &self,
        schema_id: &str,
        view_id: &str,
        fields: &mut Vec<StrTuple<'a>>,
    ) -> Result<String, String> {
        sort_fields(fields);
        let to_update: Vec<String> = fields_to_json_fields(fields);

        //[1, 1, "chat_0020cae3b...", [ "<view_id>" ], { "username": "..." }]

        let json = format!(
            r#"[{}, {}, "{}", [ "{}" ], {{ {} }} ]"#,
            self.version,
            OperationAction::UPDATE,
            schema_id,
            view_id,
            to_update.join(", ")
        );

        self.send_to_node(&json).await
    }

    pub async fn delete_instance(&self, schema_id: &str, view_id: &str) -> Result<String, String> {
        let json = format!(
            r#"[ {},{},"{}",["{}"] ]"#,
            self.version,
            OperationAction::DELETE,
            schema_id,
            view_id
        );

        self.send_to_node(&json).await
    }

    pub fn debug_print_public_key(&self) {
        let public_key = self.key_pair.public_key();
        println!("▶️ DEBUG PUB_KEY: {}", public_key);
    }

    pub async fn debug_fetch_schemas(&self) -> Result<AllSchemaDefinitionResponse, String> {
        let query = r#"query {
allSchemas: all_schema_definition_v1 {
      meta {
        documentId
        viewId
      }
      fields {
        name
        description
        fields {
          fields {
            name
            type
          }
        }
      }
    }
  }
"#;
        let result = self.client.query_unwrap(&query).await;
        let data: AllSchemaDefinitionResponse = match result {
            Ok(res) => res,
            Err(err) => return Err(format!("GraphQL error: {}", err.to_string())),
        };

        Ok(data)
    }

    async fn send_to_node(&self, json: &str) -> Result<String, String> {
        // 1. Load public key from key_pair
        let public_key = self.key_pair.public_key();

        // 2. Parse operation from JSON string
        let operation_result = serde_json::from_str(json);

        let operation: PlainOperation = match operation_result {
            Ok(op) => op,
            Err(_err) => return Err("Error at parsing JSON".to_string()),
        };

        // 3. Send `nextArgs` GraphQL query to get the arguments from the node to create the next entry
        let query = format!(
            r#"
            {{
                nextArgs(publicKey: "{}", viewId: {}) {{
                    logId
                    seqNum
                    skiplink
                    backlink
                }}
            }}
            "#,
            public_key,
            // Set `viewId` when `previous` is given in operation
            operation
                .previous()
                .map_or("null".to_owned(), |id| format!("\"{}\"", id)),
        );

        let response_result = self.client.query_unwrap::<NextArgsResponse>(&query).await;

        let response = match response_result {
            Ok(res) => res,
            Err(err) => {
                return Err(format!(
                    "GraphQL query to fetch `nextArgs` failed:\n{}",
                    err.to_string()
                ))
            }
        };

        let NextArguments {
            log_id,
            seq_num,
            skiplink,
            backlink,
        } = response.next_args;

        // 4. Create p2panda data! Encode operation, sign and encode entry
        let encoded_operation_result = encode_plain_operation(&operation);
        let encoded_operation = match encoded_operation_result {
            Ok(enc) => enc,
            Err(_err) => return Err("Could not encode operation".to_string()),
        };

        let encoded_entry_result = sign_and_encode_entry(
            &log_id,
            &seq_num,
            skiplink.as_ref(),
            backlink.as_ref(),
            &encoded_operation,
            &self.key_pair,
        );

        let encoded_entry = match encoded_entry_result {
            Ok(enc) => enc,
            Err(_err) => return Err("Could not sign and encode entry".to_string()),
        };

        let operation_id = encoded_entry.hash();
        let query = format!(
            r#"
            mutation Publish {{
                publish(entry: "{}", operation: "{}") {{
                    logId
                    seqNum
                    skiplink
                    backlink
                }}
            }}
        "#,
            encoded_entry, encoded_operation
        );

        let response_result = self.client.query_unwrap::<PublishResponse>(&query).await;
        if let Err(err) = response_result {
            return Err(format!(
                "GraphQL mutation `publish` failed:\n{}",
                err.to_string()
            ));
        }

        Ok(operation_id.to_string())
    }
}

/// Utility function to sort `Vec<StrTuple>` in alphabetical order
/// p2panda requires the fields in alphabetical order
fn sort_fields<'a>(fields: &mut Vec<StrTuple<'a>>) {
    fields.sort_by(|a, b| a.0.cmp(b.0))
}

/// Helper function to write a file.
fn write_file(path: &PathBuf, content: &str) {
    let mut file =
        File::create(path).unwrap_or_else(|_| panic!("Could not create file {:?}", path));
    write!(&mut file, "{}", content).unwrap();
}

/// Helper function to read a private key from a file, deriving a key pair from it. If it doesn't
/// exist yet, a new key pair will be generated automatically.
fn get_key_pair(path: Option<PathBuf>) -> KeyPair {
    let path = path.unwrap_or(PathBuf::from("key.txt"));

    // Read private key from file or generate a new one
    let private_key = if Path::exists(&path) {
        let key = read_to_string(path).expect("Couldn't read file!");
        key.replace('\n', "")
    } else {
        let key = hex::encode(KeyPair::new().private_key().to_bytes());
        write_file(&path, &key);
        key
    };

    // Derive key pair from private key
    KeyPair::from_private_key_str(&private_key).expect("Invalid private key")
}

/// Utility function to map a `Vec<StrTuple>` to `Vec<String>`
/// The resulting string has the shape: `"a": "b"` or `"a": b` if b is a number or boolean
fn fields_to_json_fields<'a>(fields: &Vec<StrTuple<'a>>) -> Vec<String> {
    fields
        .iter()
        .map(|(name, value)| -> String {
            let value = (**value).to_string();

            if value == "true" || value == "false" {
                return format!(r#""{}": {}"#, name, value);
            }

            if let Ok(x) = value.parse::<f64>() {
                return format!(r#""{}": {}"#, name, x);
            }

            return format!(r#""{}": "{}""#, name, value);
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::Operator;

    #[tokio::test]
    async fn create_schema_test() -> Result<(), String> {
        let op = Operator::default();

        // ---------
        // Test create schema

        let mut fields = vec![
            ("name", "str"),
            ("number", "int"),
            ("pi", "float"),
            ("isFree", "bool"),
        ];
        let schema_id = op.create_schema("test", "DESCRIPTION", &mut fields).await?;

        let schema_id = format!("test_{}", schema_id);

        let mut fields = vec![
            ("name", "UMBRA"),
            ("number", "69"),
            ("pi", "3.1416"),
            ("isFree", "false"),
        ];

        let instance_id = op.create_instance(&schema_id, &mut fields).await?;

        let mut fields = vec![
            ("name", "UMBRA_BEAR_420"),
            ("number", "10"),
            ("isFree", "true"),
        ];

        let update_id = op
            .update_instance(&schema_id, &instance_id, &mut fields)
            .await?;

        let _delete_id = op.delete_instance(&schema_id, &update_id).await?;

        // ---------

        // ---------
        // Test create pokemon schema

        let mut fields = vec![("pokemon_id", "int"), ("pokemon_name", "str")];

        let id = op
            .create_schema("POKEMON", "Pokemon schema", &mut fields)
            .await?;

        let schema_id = format!("POKEMON_{}", id);

        let mut fields = vec![("pokemon_id", "1"), ("pokemon_name", "Bulbasaur")];
        let instance_id = op.create_instance(&schema_id, &mut fields).await?;

        let mut fields = vec![("pokemon_name", "Charmander")];
        let update_id = op
            .update_instance(&schema_id, &instance_id, &mut fields)
            .await?;
        let _delete_id = op.delete_instance(&schema_id, &update_id).await?;

        let mut fields = vec![("pokemon_id", "150"), ("pokemon_name", "Mewtwo")];
        let instance_id = op.create_instance(&schema_id, &mut fields).await?;
        let _delete_id = op.delete_instance(&schema_id, &instance_id).await?;

        // ---------

        Ok(())
    }

    #[tokio::test]
    async fn debug_fetch_schemas_test() {
        let operator = Operator::default();

        operator.debug_print_public_key();
        let result = operator.debug_fetch_schemas().await;

        match result {
            Ok(data) => {
                let json = serde_json::to_string(&data).expect("Error at parsing data to json");
                println!("{}", &json);
            }
            Err(e) => panic!("{}", e),
        };
    }
}
