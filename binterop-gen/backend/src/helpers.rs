use crate::generator::Generator;
use crate::language_generators::c_gen::CGenerator;
use crate::language_generators::rust_gen::RustGenerator;
use crate::language_generators::LanguageGenerator;
use crate::optimization::{optimize_schema, SchemaOptimizations};
use crate::tokenizer::Tokenizer;
use binterop::schema::Schema;
use std::path::{Path, PathBuf};
use std::{env, fs};

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

pub fn generate_lang_files(path: &Path, gen_name: &str, schema: &Schema) -> Result<(), String> {
    match gen_name {
        "c" => {
            let mut generator = CGenerator::default();
            generator.feed(schema)?;
            generator.write(path)?;

            Ok(())
        }
        "rust" => {
            let mut generator = RustGenerator::default();
            generator.feed(schema)?;
            generator.write(path)?;

            Ok(())
        }
        _ => Err(format!("Unknown language generator name: {gen_name}")),
    }
}

pub fn language_generator(path: &Path, gen_name: &str, schema: &Schema) -> Result<(), String> {
    generate_lang_files(path, gen_name, schema)
        .map_err(|err| format!("Failed to generate language files! Error: {err}"))
}

pub fn process_text(path: &Path, text: &str) -> Result<(), String> {
    let schema = generate_schema(Some(path.into()), text, SchemaOptimizations::new())?;
    let schema_serialized = serde_json::to_string(&schema);

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

    if let Some(gen_name) = env::args()
        .filter_map(|arg| arg.strip_prefix("--gen=").map(ToString::to_string))
        .next()
    {
        language_generator(path, &gen_name, &schema)?;
        println!("\tGenerated language files using {gen_name} generator.");
    }

    Ok(())
}
