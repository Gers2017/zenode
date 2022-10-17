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
            Relation(id) => write!(f, "relation({})", id),
            RelationList(id) => write!(f, "relation_list({})", id),
            PinnedRelation(id) => write!(f, "pinned_relation({})", id),
            PinnedRelationList(id) => write!(f, "pinned_relation_list({})", id),
        }
    }
}
