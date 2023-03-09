use crate::graphql::schemas::*;
use crate::utils::get_key_pair;
use gql_client::Client;
use p2panda_rs::{
    self,
    entry::{encode::sign_and_encode_entry, traits::AsEncodedEntry},
    identity::KeyPair,
    operation::{encode::encode_plain_operation, plain::PlainOperation, traits::Actionable},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fmt::{Debug, Display};
use std::path::PathBuf;

const DEFAULT_ENDPOINT: &str = "http://localhost:2020/graphql";

pub struct Operator {
    version: usize,
    key_pair: KeyPair,
    client: Client,
}

pub fn document_fields_to_json_fields(fields: &DocumentFields) -> Vec<String> {
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

        let json = format!(
            r#"[{}, {}, "{}", [ "{}" ], {{ {} }} ]"#,
            self.version,
            OperationAction::Update,
            schema_id,
            view_id,
            payload.join(", ")
        );

        let view_id = self.send_to_node(&json).await?;
        Ok(view_id)
    }

    pub async fn delete_document(&self, schema_id: &str, view_id: &str) -> Result<String, String> {
        let json = format!(
            r#"[ {},{},"{}",["{}"] ]"#,
            self.version,
            OperationAction::Delete,
            schema_id,
            view_id
        );

        let view_id = self.send_to_node(&json).await?;
        Ok(view_id)
    }

    /// Handles p2panda operations and graphql requests
    async fn send_to_node(&self, json: &str) -> Result<String, String> {
        // 1. Load public key from key_pair
        let public_key = self.key_pair.public_key();

        // 2. Parse operation from JSON string
        let operation_result = serde_json::from_str(json);

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FieldType {
    Int,
    Float,
    Boolean,
    String,
    Relation(String),
    RelationList(String),
    PinnedRelation(String),
    PinnedRelationList(String),
}

impl Display for FieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use FieldType::*;
        match self {
            Boolean => write!(f, "bool"),
            Int => write!(f, "int"),
            Float => write!(f, "float"),
            String => write!(f, "str"),
            Relation(schema_id) => write!(f, "relation({})", schema_id),
            RelationList(schema_id) => write!(f, "relation_list({})", schema_id),
            PinnedRelation(schema_id) => write!(f, "pinned_relation({})", schema_id),
            PinnedRelationList(schema_id) => write!(f, "pinned_relation_list({})", schema_id),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FieldValue {
    Int(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Relation(String),
    RelationList(String),
    PinnedRelation(String),
    PinnedRelationList(String),
}

impl Display for FieldValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use FieldValue::*;
        match self {
            Boolean(value) => write!(f, "{}", value),
            Int(value) => write!(f, "{}", value),
            Float(value) => write!(f, "{}", value),
            // use "" on strings
            String(value) => write!(f, "\"{}\"", value),
            Relation(schema_id) => write!(f, "\"relation({})\"", schema_id),
            RelationList(schema_id) => write!(f, "\"relation_list({})\"", schema_id),
            PinnedRelation(schema_id) => write!(f, "\"pinned_relation({})\"", schema_id),
            PinnedRelationList(schema_id) => write!(f, "\"pinned_relation_list({})\"", schema_id),
        }
    }
}

type SchemaFields = HashMap<String, FieldType>;
type DocumentFields = HashMap<String, FieldValue>;

pub struct DocumentFieldBuilder {
    pub map: DocumentFields,
}

impl DocumentFieldBuilder {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn field(mut self, name: &str, value: FieldValue) -> Self {
        self.map.insert(name.to_string(), value);
        self
    }

    pub fn build(self) -> DocumentFields {
        self.map
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

        Operator::new(version, get_key_pair(key_pair_path), Client::new(endpoint))
    }
}

impl Default for OperatorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SchemaBuilder<'a> {
    operator: &'a Operator,
    name: String,
    description: String,
    map: HashMap<String, FieldType>,
}

impl<'a> SchemaBuilder<'a> {
    pub fn new(operator: &'a Operator, name: &str, description: &str) -> SchemaBuilder<'a> {
        SchemaBuilder {
            name: name.to_string(),
            description: description.to_string(),
            map: HashMap::new(),
            operator,
        }
    }

    pub fn field(mut self, name: &str, value: FieldType) -> Self {
        self.map.insert(name.to_string(), value);
        self
    }

    pub async fn build(self) -> Result<SchemaResponse<'a>, Box<dyn Error>> {
        let schema = self
            .operator
            .create_schema(&self.name, &self.description, &self.map)
            .await?;
        Ok(schema)
    }
}

pub struct SchemaResponse<'a> {
    pub id: String,
    pub name: String,
    pub fields: HashMap<String, FieldType>,
    pub operator: &'a Operator,
}

impl SchemaResponse<'_> {
    pub async fn spawn(&self, fields: &DocumentFields) -> Result<DocumentResponse, Box<dyn Error>> {
        let document = self
            .operator
            .create_document(&self.get_schema_id(), fields)
            .await?;
        Ok(document)
    }

    pub fn get_schema_id(&self) -> String {
        format!("{}_{}", self.name, self.id)
    }

    pub fn find_by_id(&self, view_id: &str) -> DocumentResponse {
        todo!("Not implemented yet");
    }
    pub fn find_many(&self, take: usize, skip: usize) -> Vec<DocumentResponse> {
        todo!("Not implemented yet");
    }
}

pub struct DocumentResponse<'a> {
    pub id: String,
    pub schema_id: String,
    pub fields: HashMap<String, FieldValue>,
    pub operator: &'a Operator,
}

impl<'a> DocumentResponse<'a> {
    pub fn new(
        id: &str,
        schema_id: &str,
        fields: HashMap<String, FieldValue>,
        operator: &'a Operator,
    ) -> Self {
        Self {
            id: id.to_string(),
            schema_id: schema_id.to_string(),
            fields,
            operator,
        }
    }

    pub async fn update_field(
        &self,
        name: &str,
        value: FieldValue,
    ) -> Result<String, Box<dyn Error>> {
        let fields = DocumentFieldBuilder::new().field(name, value).build();

        let view_id = self
            .operator
            .update_document(&self.schema_id, &self.id, &fields)
            .await?;

        Ok(view_id)
    }

    pub async fn update(&self, fields: DocumentFields) -> Result<String, Box<dyn Error>> {
        let view_id = self
            .operator
            .update_document(&self.schema_id, &self.id, &fields)
            .await?;
        Ok(view_id)
    }

    pub async fn delete(&self) -> Result<String, Box<dyn Error>> {
        let view_id = self
            .operator
            .delete_document(&self.schema_id, &self.id)
            .await?;
        Ok(view_id)
    }
}

pub struct DocumentBuilder<'a> {
    schema_response: &'a SchemaResponse<'a>,
    map: HashMap<String, FieldValue>,
}

impl<'a> DocumentBuilder<'a> {
    pub fn new(schema_response: &'a SchemaResponse) -> DocumentBuilder<'a> {
        Self {
            map: HashMap::new(),
            schema_response,
        }
    }

    pub fn field(mut self, key: &str, value: FieldValue) -> Self {
        self.map.insert(key.to_string(), value);
        self
    }

    pub async fn build(self) -> Result<DocumentResponse<'a>, Box<dyn Error>> {
        let document = self.schema_response.spawn(&self.map).await?;
        Ok(document)
    }
}
