use std::fmt::Display;

pub enum FieldType<'a> {
    Bool,
    Int,
    Float,
    Str,
    Relation(&'a str),
    RelationList(&'a str),
    PinnedRelation(&'a str),
    PinnedRelationList(&'a str),
}

impl Display for FieldType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use FieldType::*;
        match self {
            Bool => write!(f, "bool"),
            Int => write!(f, "int"),
            Float => write!(f, "float"),
            Str => write!(f, "str"),
            Relation(schema_id) => write!(f, "relation({})", schema_id),
            RelationList(schema_id) => write!(f, "relation_list({})", schema_id),
            PinnedRelation(schema_id) => write!(f, "pinned_relation({})", schema_id),
            PinnedRelationList(schema_id) => write!(f, "pinned_relation_list({})", schema_id),
        }
    }
}
