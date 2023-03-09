use serde::{Deserialize, Serialize};
use std::fmt::Display;

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
