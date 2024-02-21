mod generator;
mod tokenizer;

use crate::generator::Generator;
use crate::tokenizer::Tokenizer;
use binterop::schema::Schema;
use std::path::PathBuf;
use std::{env, fs};

fn generate_schema(definition_text: &str) -> Result<Schema, String> {
    let mut tokenizer = Tokenizer::new(definition_text);
    let mut generator = Generator::default();

    while let Some(token) = tokenizer.yield_token()? {
        generator.feed(token)?
    }

    Ok(generator.get_schema())
}

fn main() {
    let mut args_iter = env::args();
    args_iter.next();

    for path in args_iter.map(PathBuf::from) {
        match fs::read_to_string(&path) {
            Ok(file_text) => {
                let schema = match generate_schema(&file_text) {
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
            }
            Err(err) => eprintln!("{path:?}: {err:?}"),
        }
    }
}
