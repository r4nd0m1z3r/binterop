mod generator;
mod language_generators;
mod tokenizer;

use crate::generator::Generator;
use crate::language_generators::c_gen::CGenerator;
use crate::language_generators::LanguageGenerator;
use crate::tokenizer::Tokenizer;
use binterop::schema::Schema;
use std::path::PathBuf;
use std::{env, fs};

fn generate_schema(file_path: Option<PathBuf>, definition_text: &str) -> Result<Schema, String> {
    let mut tokenizer = Tokenizer::new(file_path, definition_text);
    let mut generator = Generator::default();

    while let Some(token) = tokenizer.yield_token()? {
        generator.feed(token)?
    }

    Ok(generator.output())
}

fn generate_lang_files(gen_name: &str, schema: &Schema) -> Result<(String, String), String> {
    match gen_name {
        "c" => {
            let mut generator = CGenerator::default();
            generator.feed(schema)?;

            Ok((generator.output_extension(), generator.output()))
        }
        _ => Err(format!("Unknown language generator name: {gen_name}")),
    }
}

fn main() {
    let args_iter = env::args();

    for path in args_iter
        .skip(1)
        .map(PathBuf::from)
        .flat_map(fs::canonicalize)
    {
        match fs::read_to_string(&path) {
            Ok(file_text) => {
                let schema = match generate_schema(Some(path.clone()), &file_text) {
                    Ok(schema) => schema,
                    Err(err) => {
                        eprintln!("{path:?}: {err}");
                        continue;
                    }
                };

                let schema_serialized = serde_json::to_string(&schema);

                match schema_serialized {
                    Ok(data) => {
                        let output_path = path.with_extension("json");
                        if let Err(err) = fs::write(&output_path, data) {
                            eprintln!("{path:?}: Failed to write serialized schema to {output_path:?}! Error: {err:?}");
                        } else {
                            println!("{path:?} -> {output_path:?}");
                        }
                    }
                    Err(err) => eprintln!("{path:?}: Failed to serialize schema! Error: {err:?}"),
                }

                if let Some(gen_name) = env::args()
                    .filter_map(|arg| arg.strip_prefix("--gen=").map(ToString::to_string))
                    .next()
                {
                    match generate_lang_files(&gen_name, &schema) {
                        Ok((ext, output)) => match fs::write(path.with_extension(ext), output) {
                            Ok(_) => println!(
                                "{path:?}: Generated language files using {gen_name} generator."
                            ),
                            Err(err) => eprintln!(
                                "{path:?}: Failed to write generated language file! Error: {err}"
                            ),
                        },
                        Err(err) => {
                            eprintln!("{path:?}: Failed to generate language files! Error: {err}");
                        }
                    }
                }
            }
            Err(err) => eprintln!("{path:?}: {err:?}"),
        }
        println!()
    }
}
