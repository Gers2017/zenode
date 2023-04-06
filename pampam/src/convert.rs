use zenode::{FieldType, FieldValue};

use crate::{
    parser::{self, parse_relation},
    PamError,
};

type TypeFieldTuple = (String, FieldType);

pub fn convert_to_type_fields(
    fields: &[(String, String)],
) -> Result<Vec<TypeFieldTuple>, PamError> {
    fields
        .iter()
        .map(|(ident, typ)| match_type_field(typ).map(|f_type| (ident.to_string(), f_type)))
        .collect()
}

pub fn match_type_field(typ: &str) -> Result<FieldType, PamError> {
    match typ {
        "str" => Ok(FieldType::String),
        "int" => Ok(FieldType::Int),
        "float" => Ok(FieldType::Float),
        "bool" => Ok(FieldType::Boolean),
        s => {
            let res = parser::parse_relation(s)?;

            if s.starts_with("relation(") {
                Ok(FieldType::Relation(res.document_id))
            } else if s.starts_with("relation_list(") {
                Ok(FieldType::RelationList(res.document_id))
            } else if s.starts_with("pinned_relation(") {
                Ok(FieldType::PinnedRelation(res.document_id))
            } else if s.starts_with("pinned_relation_list(") {
                Ok(FieldType::PinnedRelationList(res.document_id))
            } else {
                Err(PamError::UnknownTypeError {
                    typ: typ.to_string(),
                })
            }
        }
    }
}

type ValueFieldTuple = (String, FieldValue);

pub fn convert_to_value_fields(
    fields: &[(String, String)],
) -> Result<Vec<ValueFieldTuple>, PamError> {
    fields
        .iter()
        .map(|(ident, value)| match_value_field(value).map(|x| (ident.to_string(), x)))
        .collect::<Result<Vec<_>, PamError>>()
}

pub fn match_value_field(value: &str) -> Result<FieldValue, PamError> {
    match value {
        "true" => Ok(FieldValue::Boolean(true)),
        "false" => Ok(FieldValue::Boolean(false)),
        value => {
            if let Ok(number) = value.parse::<f64>() {
                let has_dot = value.contains('.');
                if has_dot {
                    return Ok(FieldValue::Float(number));
                } else {
                    return Ok(FieldValue::Int(number as i64));
                }
            }

            if value.starts_with("relation") || value.starts_with("pinned_relation") {
                let res = parse_relation(value)?;
                return match res.relation_name.as_str() {
                    "relation" => Ok(FieldValue::Relation(res.document_id)),
                    "relation_list" => Ok(FieldValue::RelationList(res.document_id)),
                    "pinned_relation" => Ok(FieldValue::PinnedRelation(res.document_id)),
                    "pinned_relation_list" => Ok(FieldValue::PinnedRelationList(res.document_id)),
                    _ => Err(PamError::UnknownTypeError {
                        typ: res.relation_name.to_string(),
                    }),
                };
            } else {
                Ok(FieldValue::String(value.to_string()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_to_type_fields() {
        let fields = vec![
            ("foo".to_string(), "int".to_string()),
            ("bar".to_string(), "bool".to_string()),
            ("buzz".to_string(), "relation_list(id_123)".to_string()),
        ];

        let res = convert_to_type_fields(&fields);

        assert!(res.is_ok());
        let types = res.unwrap();

        assert_eq!(types.get(0), Some(&("foo".to_string(), FieldType::Int)));
        assert_eq!(types.get(1), Some(&("bar".to_string(), FieldType::Boolean)));
        assert_eq!(
            types.get(2),
            Some(&(
                "buzz".to_string(),
                FieldType::RelationList("id_123".to_string())
            ))
        );
    }

    #[test]
    fn test_convert_to_value_fields() {
        let fields = vec![
            ("foo".to_string(), "100".to_string()),
            ("bar".to_string(), "true".to_string()),
            ("buzz".to_string(), "relation_list([id_123])".to_string()),
        ];

        let res = convert_to_value_fields(&fields);
        assert!(res.is_ok());

        let values = res.unwrap();

        assert_eq!(
            values.get(0),
            Some(&("foo".to_string(), FieldValue::Int(100)))
        );
        assert_eq!(
            values.get(1),
            Some(&("bar".to_string(), FieldValue::Boolean(true)))
        );
        assert_eq!(
            values.get(2),
            Some(&(
                "buzz".to_string(),
                FieldValue::RelationList("[id_123]".to_string())
            ))
        );
    }
}
