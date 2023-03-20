use crate::document::{DocumentFields, DocumentResponse};
use crate::schema::SchemaFields;
use crate::utils::get_key_pair;
use crate::{graphql::schemas::*, schema::SchemaResponse};
use gql_client::Client;
use p2panda_rs::{
    self,
    entry::{encode::sign_and_encode_entry, traits::AsEncodedEntry},
    identity::KeyPair,
    operation::{encode::encode_plain_operation, plain::PlainOperation, traits::Actionable},
};
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use std::fmt::{Debug, Display};
use std::path::PathBuf;

const DEFAULT_ENDPOINT: &str = "http://localhost:2020/graphql";

pub struct Operator {
    pub version: usize,
    pub key_pair: KeyPair,
    pub client: Client,
}

fn document_fields_to_json_fields(fields: &DocumentFields) -> Vec<String> {
    let mut keys: Vec<_> = fields.keys().collect();
    keys.sort_by(|a, b| a.cmp(b));

    let result: Vec<_> = keys
        .iter()
        .map(|key| {
            let value = fields.get(&key.to_string()).unwrap().to_string();
            format!("\"{}\": {}", &key, &value)
        })
        .collect();
    result
}

impl Operator {
    pub fn new(version: usize, key_pair: KeyPair, client: Client) -> Self {
        Self {
            version,
            key_pair,
            client,
        }
    }

    /// Creates a new Operator from scratch
    pub fn builder() -> OperatorBuilder {
        OperatorBuilder::new()
    }

    /// Creates a schema by first publishing the fields, retrieving the field ids
    /// and publishing the schema with the field ids
    pub async fn create_schema(
        &self,
        name: &str,
        description: &str,
        fields: &SchemaFields,
    ) -> Result<SchemaResponse, Box<dyn Error>> {
        // publish fields to node and retrieve field_ids
        let field_ids = self.publish_fields(&fields).await?;

        // create schema with field_ids
        let schema = self
            .publish_schema(name, description, &field_ids, &fields)
            .await?;
        Ok(schema)
    }

    /// Publishes the schema definition to the node
    async fn publish_schema(
        &self,
        name: &str,
        description: &str,
        field_ids: &[String],
        fields: &SchemaFields,
    ) -> Result<SchemaResponse, String> {
        let field_content: String = field_ids
            .iter()
            .map(|it| format!("[\"{}\"]", it))
            .collect::<Vec<_>>()
            .join(", ");
        /*
            "fields": [
              ["<field_id>"],
              ["<field_id>"]
            ],
        */
        let json_data = format!(
            r#"[{}, {}, "schema_definition_v1", {{ "description": "{}", "fields": [{}], "name": "{}" }}]"#,
            self.version,
            OperationAction::Create,
            description,
            field_content,
            name
        );

        let id = self.send_to_node(&json_data).await?;
        Ok(SchemaResponse {
            id,
            name: name.to_string(),
            operator: self,
            fields: fields.clone(),
        })
    }

    /// Publishes the field definitions to the node
    async fn publish_fields(&self, fields: &SchemaFields) -> Result<Vec<String>, String> {
        let mut keys: Vec<_> = fields.keys().collect();
        keys.sort_by(|a, b| a.cmp(b));

        let mut ids: Vec<String> = Vec::with_capacity(fields.len());

        for key in keys.iter() {
            let json_data = format!(
                r#"[{}, {}, "schema_field_definition_v1", {{ "name": "{}", "type": "{}" }}]"#,
                self.version,
                OperationAction::Create,
                key,
                fields.get(&key.to_string()).unwrap()
            );

            let id = self.send_to_node(&json_data).await?;
            ids.push(id);
        }

        Ok(ids)
    }

    /// Creates an document following the shape of the schema with the respective schema_id
    pub async fn create_document(
        &self,
        schema_id: &str,
        fields: &DocumentFields,
    ) -> Result<DocumentResponse, String> {
        let payload: Vec<_> = document_fields_to_json_fields(&fields);

        // [1, 0, "chat_0020cae3b...", {"msg": "...", "username": "..." } ]

        let json_data = format!(
            r#"[{}, {}, "{}", {{ {} }} ]"#,
            self.version,
            OperationAction::Create,
            schema_id,
            payload.join(", ")
        );

        dbg!(&json_data);

        let id = self.send_to_node(&json_data).await?;
        Ok(DocumentResponse {
            id,
            schema_id: schema_id.to_string(),
            fields: fields.clone(),
            operator: self,
        })
    }

    pub async fn update_document(
        &self,
        schema_id: &str,
        view_id: &str,
        fields: &DocumentFields,
    ) -> Result<String, String> {
        let payload: Vec<_> = document_fields_to_json_fields(&fields);

        //[1, 1, "chat_0020cae3b...", [ "<view_id>" ], { "username": "..." }]

        let json_data = format!(
            r#"[{}, {}, "{}", [ "{}" ], {{ {} }} ]"#,
            self.version,
            OperationAction::Update,
            schema_id,
            view_id,
            payload.join(", ")
        );

        let view_id = self.send_to_node(&json_data).await?;
        Ok(view_id)
    }

    pub async fn delete_document(&self, schema_id: &str, view_id: &str) -> Result<String, String> {
        let json_data = format!(
            r#"[ {},{},"{}",["{}"] ]"#,
            self.version,
            OperationAction::Delete,
            schema_id,
            view_id
        );

        let view_id = self.send_to_node(&json_data).await?;
        Ok(view_id)
    }

    /// Handles p2panda operations and graphql requests
    async fn send_to_node(&self, json_data: &str) -> Result<String, String> {
        // 1. Load public key from key_pair
        let public_key = self.key_pair.public_key();

        // 2. Parse operation from JSON string
        let operation_result = serde_json::from_str(json_data);

        let operation: PlainOperation = match operation_result {
            Ok(op) => op,
            Err(err) => return Err(err.to_string()),
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
}

impl Default for Operator {
    fn default() -> Self {
        let endpoint = env::var("ENDPOINT").ok();
        let mut operator = OperatorBuilder::new();

        if let Some(endpoint) = endpoint {
            operator = operator.endpoint(&endpoint);
        }

        operator.build()
    }
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

    pub fn key_pair_path(mut self, path: PathBuf) -> Self {
        self.key_pair_path = Some(path);
        self
    }

    pub fn endpoint(mut self, endpoint: &str) -> Self {
        self.endpoint = endpoint.to_string();
        self
    }

    pub fn build(self) -> Operator {
        let Self {
            version,
            key_pair_path,
            endpoint,
        } = self;

        let key_pair = get_key_pair(key_pair_path);
        Operator::new(version, key_pair, Client::new(endpoint))
    }
}

impl Default for OperatorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[repr(u8)]
enum OperationAction {
    Create = 0,
    Update = 1,
    Delete = 2,
}

impl Display for OperationAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", *self as u8)
    }
}
