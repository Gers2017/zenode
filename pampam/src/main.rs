use clap::Parser;
use pampam::{
    convert::{convert_to_type_fields, convert_to_value_fields},
    parser::{parse_fields, validate_type_fields},
    Commands, CreateCommands, DeleteCommands, PamPamCli, UpdateCommands,
};
use std::error::Error;
use zenode::{document::DocumentFieldBuilder, schema::SchemaFieldBuilder, Operator};

#[tokio::main]
async fn main() {
    let cli = PamPamCli::parse();

    let mut op_builder = Operator::builder();
    if let Some(x) = cli.endpoint {
        op_builder = op_builder.endpoint(&x);
    }
    if let Some(x) = cli.key_pair_path {
        op_builder = op_builder.key_pair_path(x);
    }
    if let Some(x) = cli.op_version {
        op_builder = op_builder.version(x);
    }

    let operator = op_builder.build();

    if let Err(e) = handle_commands(&operator, &cli.commands).await {
        eprintln!("{}", e);
    }
}

pub async fn handle_commands(
    operator: &Operator,
    commands: &Commands,
) -> Result<(), Box<dyn Error>> {
    match commands {
        Commands::Create(CreateCommands::Schema {
            name,
            description,
            fields,
        }) => {
            let fields = parse_fields(fields)?;
            validate_type_fields(&fields)?;

            let type_fields = convert_to_type_fields(&fields)?;

            let mut schema_fields = SchemaFieldBuilder::new();

            for (ident, typ) in type_fields {
                schema_fields = schema_fields.field(&ident, typ);
            }

            dbg!(&schema_fields.map);

            let res = operator
                .create_schema(name, description, &schema_fields.build())
                .await?;

            println!("schema_id: {}  schema name: {}", &res.id, &res.name);
        }
        Commands::Create(CreateCommands::Document { schema_id, fields }) => {
            let fields = parse_fields(fields)?;
            let value_fields = convert_to_value_fields(&fields)?;

            let mut document_fields = DocumentFieldBuilder::new();

            for (k, v) in value_fields {
                document_fields = document_fields.field(&k, v);
            }

            dbg!(&document_fields.map);

            let res = operator
                .create_document(schema_id, &document_fields.build())
                .await?;

            println!("document_id: {}", &res.id);
        }
        Commands::Update(UpdateCommands::Document {
            schema_id,
            view_id,
            fields,
        }) => {
            let fields = parse_fields(fields)?;
            let value_fields = convert_to_value_fields(&fields)?;

            let mut document_fields = DocumentFieldBuilder::new();

            for (k, v) in value_fields {
                document_fields = document_fields.field(&k, v);
            }

            dbg!(&document_fields.map);

            let update_id = operator
                .update_document(schema_id, view_id, &document_fields.build())
                .await?;

            println!("updated document_id: {}", &update_id);
        }
        Commands::Delete(DeleteCommands::Document { schema_id, view_id }) => {
            let delete_id = operator.delete_document(schema_id, view_id).await?;

            println!("deleted document_id: {}", &delete_id);
        }
    };

    Ok(())
}