use crate::generator::Generator;
use crate::language_generators::nim::NimLanguageGenerator;
use crate::language_generators::rust::RustLanguageGenerator;
use crate::language_generators::{LanguageGenerator, LanguageGeneratorState};
use crate::optimization::{optimize_schema, SchemaOptimizations};
use crate::tokenizer::Tokenizer;
use binterop::schema::Schema;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

pub fn generate_schema(
    file_path: Option<PathBuf>,
    definition_text: &str,
    optimizations: SchemaOptimizations,
) -> Result<Schema, String> {
    let mut tokenizer = Tokenizer::new(file_path, definition_text);
    let mut generator = Generator::default();

    while let Some(token) = tokenizer.yield_token()? {
        generator.feed(token)?
    }
    let mut schema = generator.output();

    optimize_schema(&mut schema, optimizations);

    Ok(schema)
}

pub fn serialize_schema(schema: &Schema) -> Result<String, serde_json::Error> {
    serde_json::to_string(&schema)
}

pub fn generate_lang_files(
    bintdef_path: &Path,
    gen_name: &str,
    schema: &Schema,
) -> Result<(), String> {
    let file_name = bintdef_path
        .file_name()
        .map(OsStr::to_str)
        .unwrap()
        .unwrap();
    let mut state = LanguageGeneratorState::new(file_name, schema);

    match gen_name {
        "rust" => {
            let mut generator = RustLanguageGenerator::default();
            generator.generate(&mut state, bintdef_path.parent().unwrap())?;

            Ok(())
        }
        "nim" => {
            let mut generator = NimLanguageGenerator::default();
            generator.generate(&mut state, bintdef_path.parent().unwrap())?;

            Ok(())
        }
        _ => Err(format!("Unknown language generator name: {gen_name}")),
    }
    .map_err(|err| format!("Failed to generate language files! Error: {err}"))
}

pub fn process_text(path: &Path, text: &str, args: &[String]) -> Result<(), String> {
    let schema = generate_schema(Some(path.into()), text, SchemaOptimizations::default())?;
    let schema_serialized = serialize_schema(&schema);

    match schema_serialized {
        Ok(data) => {
            let output_path = path.with_extension("json");
            fs::write(&output_path, data).map_err(|err| {
                format!("Failed to write serialized schema to {output_path:?}! Error: {err:?}")
            })?;
            println!("\tSchema written to {output_path:?}");
        }
        Err(err) => Err(format!(
            "{path:?}: Failed to serialize schema! Error: {err:?}"
        ))?,
    }

    if let Some(gen_name) = args
        .iter()
        .filter_map(|arg| arg.strip_prefix("--gen=").map(ToString::to_string))
        .next()
    {
        generate_lang_files(path, &gen_name, &schema)?;
        println!("\tGenerated language files using {gen_name} generator.");
    }

    Ok(())
}
