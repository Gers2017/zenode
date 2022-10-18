use crate::builder::fields::FieldType;
use crate::graphql::{self, schemas::*};
use crate::utils::*;

use gql_client::Client;
use p2panda_rs::{
    self,
    entry::{encode::sign_and_encode_entry, traits::AsEncodedEntry},
    identity::KeyPair,
    operation::{encode::encode_plain_operation, plain::PlainOperation, traits::Actionable},
};
use std::fmt::{Debug, Display};
use std::path::PathBuf;

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
enum OperationAction {
    Create = 0,
    Update = 1,
    Delete = 2,
}

impl Display for OperationAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", *self as usize)
    }
}

pub type StringTuple = (String, String);

pub fn field(a: &str, b: &str) -> StringTuple {
    (a.to_string(), b.to_string())
}

pub fn field_def(name: &str, field_type: FieldType) -> StringTuple {
    (name.to_string(), field_type.to_string())
}

const DEFAULT_ENDPOINT: &str = "http://localhost:2020/graphql";

pub struct Operator {
    version: usize,
    key_pair: KeyPair,
    client: Client,
}

pub struct OperatorBuilder {
    version: usize,
    key_pair_path: Option<PathBuf>,
    endpoint: String,
}

impl OperatorBuilder {
    pub fn new() -> Self {
        OperatorBuilder {
            version: 1,
            key_pair_path: None,
            endpoint: DEFAULT_ENDPOINT.to_string(),
        }
    }

    pub fn version(mut self, v: usize) -> Self {
        self.version = v;
        self
    }

    pub fn key_pair_path(mut self, path: Option<PathBuf>) -> Self {
        self.key_pair_path = path;
        self
    }

    pub fn endpoint(mut self, endpoint: Option<String>) -> Self {
        if endpoint.is_some() {
            self.endpoint = endpoint.map(|x| x.to_string()).unwrap();
        }

        self
    }

    pub fn build(self) -> Operator {
        let Self {
            version,
            key_pair_path,
            endpoint,
        } = self;

        Operator {
            version: version,
            key_pair: get_key_pair(key_pair_path),
            client: Client::new(endpoint),
        }
    }
}

impl Operator {
    /// Creates a new Operator from scratch
    pub fn new() -> OperatorBuilder {
        OperatorBuilder::new()
    }

    /// Creates a new Operator with default values
    /// `version: 1, path: "key.txt", endpoint: ENDPOINT env variable or if unset "http://localhost:2020/graphql"`
    pub fn default() -> Self {
        let endpoint = std::env::var("ENDPOINT").ok();
        Operator::new().endpoint(endpoint).build()
    }

    /// Creates a schema by first publishing the fields, retrieving the field ids
    /// and publishing the schema with the field ids
    pub async fn create_schema(
        &self,
        name: &str,
        description: &str,
        fields: &mut [StringTuple],
    ) -> Result<String, String> {
        // publish fields to node and retrieve field_ids
        let field_ids = self.publish_fields(fields).await?;

        // create schema with field_ids
        self.publish_schema(name, description, &field_ids).await
    }

    /// Publishes the schema definition to the node
    async fn publish_schema(
        &self,
        name: &str,
        description: &str,
        field_ids: &[String],
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
            OperationAction::Create,
            description,
            field_content,
            name
        );

        self.send_to_node(&json).await
    }

    /// Publishes the field definitions to the node
    async fn publish_fields(&self, fields: &mut [StringTuple]) -> Result<Vec<String>, String> {
        sort_fields(fields);

        let mut field_ids: Vec<String> = Vec::with_capacity(fields.len());

        for (name, f_type) in fields.iter() {
            let json = format!(
                r#"[{}, {}, "schema_field_definition_v1", {{ "name": "{}", "type": "{}" }}]"#,
                self.version,
                OperationAction::Create,
                name,
                f_type
            );

            let id = self.send_to_node(&json).await?;
            field_ids.push(id);
        }

        Ok(field_ids)
    }

    /// Creates an instance following the shape of the schema with the respective schema_id
    pub async fn create_instance(
        &self,
        schema_id: &str,
        fields: &mut [StringTuple],
    ) -> Result<String, String> {
        sort_fields(fields);
        let payload_content: Vec<String> = fields_to_json_fields(fields);

        // [1, 0, "chat_0020cae3b...", {"msg": "...", "username": "..." } ]

        let json = format!(
            r#"[{}, {}, "{}", {{ {} }} ]"#,
            self.version,
            OperationAction::Create,
            schema_id,
            payload_content.join(", ")
        );

        self.send_to_node(&json).await
    }

    /// Updates partially or completely an instance with the respective view_id
    pub async fn update_instance(
        &self,
        schema_id: &str,
        view_id: &str,
        fields: &mut [StringTuple],
    ) -> Result<String, String> {
        sort_fields(fields);
        let to_update: Vec<String> = fields_to_json_fields(fields);

        //[1, 1, "chat_0020cae3b...", [ "<view_id>" ], { "username": "..." }]

        let json = format!(
            r#"[{}, {}, "{}", [ "{}" ], {{ {} }} ]"#,
            self.version,
            OperationAction::Update,
            schema_id,
            view_id,
            to_update.join(", ")
        );

        self.send_to_node(&json).await
    }

    /// Deletes an instance with the respective view_id
    pub async fn delete_instance(&self, schema_id: &str, view_id: &str) -> Result<String, String> {
        let json = format!(
            r#"[ {},{},"{}",["{}"] ]"#,
            self.version,
            OperationAction::Delete,
            schema_id,
            view_id
        );

        self.send_to_node(&json).await
    }

    /// Handles p2panda operations and graphql requests
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
                    err
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
            return Err(format!("GraphQL mutation `publish` failed:\n{}", err));
        }

        Ok(operation_id.to_string())
    }

    pub fn debug_print_public_key(&self) {
        let public_key = self.key_pair.public_key();
        println!("▶️ DEBUG PUB_KEY: {}", public_key);
    }

    /// Fetches all the schema definitions returning `AllSchemaDefinitionResponse` or `String` on error
    pub async fn get_all_schema_definition(&self) -> Result<AllSchemaDefinitionResponse, String> {
        let query = graphql::queries::get_all_schemas_query;
        let data: AllSchemaDefinitionResponse = match self.client.query_unwrap(query).await {
            Ok(data) => data,
            Err(err) => return Err(err.to_string()),
        };

        Ok(data)
    }

    pub async fn get_schema_definition(
        &self,
        document_id: &str,
        view_id: &str,
    ) -> Result<SchemaDefinitionResponse, String> {
        let query = graphql::queries::get_schema_query;
        let vars = GetSchemaVars {
            id: document_id.to_string(),
            view_id: view_id.to_string(),
        };

        let data = match self.client.query_with_vars_unwrap(query, vars).await {
            Ok(data) => data,
            Err(err) => return Err(err.to_string()),
        };

        Ok(data)
    }

    pub async fn print_all_schemas(&self) -> Result<(), String> {
        let data = self.get_all_schema_definition().await?;
        serde_json::to_string_pretty(&data).map_err(|err| err.to_string())?;
        Ok(())
    }

    pub async fn print_schema(&self, document_id: &str, view_id: &str) -> Result<(), String> {
        let data = self.get_schema_definition(document_id, view_id).await?;
        serde_json::to_string_pretty(&data).map_err(|err| err.to_string())?;
        Ok(())
    }
}
