use nom::{bytes::complete::*, IResult};

use crate::PamError;

type StrTuple = (String, String);

pub fn parse_fields(fields: &[String]) -> Result<Vec<StrTuple>, PamError> {
    fields
        .iter()
        .map(|s| {
            let v = s.split(':').collect::<Vec<_>>();

            match v[..] {
                [x, y] => {
                    let f = (x.trim().to_string(), y.trim().to_lowercase());
                    Ok(f)
                }
                [_] => Err(PamError::ParseError {
                    field: s.to_string(),
                    reason: format!("Error missing \":\" in field: \"{}\"", &s),
                }),
                _ => Err(PamError::ParseError {
                    field: s.to_string(),
                    reason: format!("Error more than one \":\" in field: \"{}\"", &s),
                }),
            }
        })
        .collect()
}

pub fn validate_type_fields(type_fields: &[StrTuple]) -> Result<(), PamError> {
    type_fields.iter().map(validate_type_field).collect()
}

pub fn validate_type_field((_ident, typ): &StrTuple) -> Result<(), PamError> {
    match typ.as_str() {
        "str" => Ok(()),
        "int" => Ok(()),
        "float" => Ok(()),
        "bool" => Ok(()),
        s => {
            if (s.starts_with("relation(")
                || s.starts_with("relation_list(")
                || s.starts_with("pinned_relation(")
                || s.starts_with("pinned_relation_list("))
                && s.ends_with(')')
            {
                Ok(())
            } else {
                Err(PamError::UnknownTypeError {
                    typ: typ.to_string(),
                })
            }
        }
    }
}

#[cfg(test)]
mod parse_tests {

    use super::*;

    #[test]
    fn test_parse_fields() -> Result<(), PamError> {
        let fields = vec![
            "name:str",
            "id: int",
            "is_ok : bool",
            "  score : float  ",
            "link: relation(id_000123123)",
        ];

        let fields: Vec<_> = fields.iter().map(|s| s.to_string()).collect();
        let fields = parse_fields(&fields)?;

        assert_eq!(
            fields.get(0),
            Some(&("name".to_string(), "str".to_string()))
        );
        assert_eq!(fields.get(1), Some(&("id".to_string(), "int".to_string())));
        assert_eq!(
            fields.get(2),
            Some(&("is_ok".to_string(), "bool".to_string()))
        );
        assert_eq!(
            fields.get(3),
            Some(&("score".to_string(), "float".to_string()))
        );
        assert_eq!(
            fields.get(4),
            Some(&("link".to_string(), "relation(id_000123123)".to_string()))
        );

        Ok(())
    }

    #[test]
    fn test_validate_type_fields() -> Result<(), PamError> {
        let fields = vec![
            "name:str",
            "id: int",
            "is_ok : bool",
            "  score : float  ",
            "link: relation(id_000123123)",
        ];

        let fields: Vec<_> = fields.iter().map(|s| s.to_string()).collect();
        let fields = parse_fields(&fields)?;

        for f in fields.iter() {
            validate_type_field(&f)?;
        }

        Ok(())
    }

    #[test]
    fn test_missing_err() {
        let fields = vec![String::from("name    str")];
        let res = parse_fields(&fields);
        assert!(res.is_err());
    }

    #[test]
    fn test_too_many_err() {
        let fields = vec![String::from(":  :: name  :str")];
        let res = parse_fields(&fields);
        assert!(res.is_err());
    }
}

#[derive(Debug)]
pub struct RelationData {
    pub relation_name: String,
    pub document_id: String,
}

// input: PinnedRelationList(DocumentId) -> { "PinnedRelationList", "DocumentId" }
// input: RelationList(DocumentId) -> { "RelationList", "DocumentId" }
fn nom_parse_relation(input: &str) -> IResult<&str, RelationData> {
    let input = input.trim();
    let (input, relation) = take_till(|c| c == '(')(input)?;

    let (input, _) = tag("(")(input)?;

    let (input, document_id) = take_till(|c| c == ')')(input)?;

    let (input, _) = tag(")")(input)?;

    Ok((
        input,
        RelationData {
            relation_name: relation.to_string(),
            document_id: document_id.to_string(),
        },
    ))
}

pub fn parse_relation(input: &str) -> Result<RelationData, PamError> {
    let input_og = input.to_string();

    nom_parse_relation(input)
        .map(|(_input, result)| result)
        .map_err(|err| PamError::ParseError {
            field: input_og,
            reason: err.to_string(),
        })
}

#[cfg(test)]
mod relation_tests {
    use super::{parse_relation, PamError};

    #[test]
    fn test_parse_relation() {
        let input = String::from("   pinned_relation_list(id_1232343232)");
        match parse_relation(&input) {
            Ok(result) => {
                assert_eq!(result.relation_name, String::from("pinned_relation_list"));
                assert_eq!(result.document_id, String::from("id_1232343232"));
            }
            Err(e) => eprintln!("{}", e),
        };
    }

    #[test]
    fn test_parse_relation_err() -> Result<(), PamError> {
        let input = String::from("   relation_list(((((id_1232343232)))))  ");
        parse_relation(&input)?;
        Ok(())
    }
}
