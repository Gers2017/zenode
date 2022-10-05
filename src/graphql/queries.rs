#[allow(non_upper_case_globals)]
pub const get_all_schemas_query: &str = r#"query {
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

#[allow(non_upper_case_globals)]
pub const get_schema_query: &str = r#"query Schema($id: DocumentId!, $viewId: DocumentViewId!) {
  schema: schema_definition_v1(id: $id, viewId: $viewId) {
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
}"#;
