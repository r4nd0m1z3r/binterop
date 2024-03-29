mod generator;
mod language_generators;
mod tokenizer;

use crate::generator::Generator;
use crate::language_generators::c_gen::CGenerator;
use crate::language_generators::LanguageGenerator;
use crate::tokenizer::Tokenizer;
use binterop::schema::Schema;
use std::path::{Path, PathBuf};
use std::{env, fs};

#[derive(Copy, Clone)]
struct SchemaOptimizations {
    data_type_layout: bool,
    add_padding: bool,
}
impl SchemaOptimizations {
    fn new() -> Self {
        let args = env::args().collect::<Vec<_>>();

        Self {
            data_type_layout: !args.contains(&"--dont-optimize-layout".to_string()),
            add_padding: !args.contains(&"--dont-add-padding".to_string()),
        }
    }
}

fn optimize_data_type_layouts(schema: &mut Schema) {
    let mut field_sizes = schema
        .types
        .iter()
        .flat_map(|data_type| data_type.fields.iter().map(|field| field.size(schema)))
        .collect::<Vec<_>>();
    let mut field_sizes_cursor = 0;

    for data_type in &mut schema.types {
        let field_sizes =
            &mut field_sizes[field_sizes_cursor..field_sizes_cursor + data_type.fields.len()];
        field_sizes_cursor += data_type.fields.len();

        let mut permutation = permutation::sort_unstable_by(&field_sizes, |f1, f2| f1.cmp(f2));
        permutation.apply_slice_in_place(&mut data_type.fields);
        permutation.apply_slice_in_place(field_sizes);

        let mut field_offset = 0;
        for (field, &size) in data_type.fields.iter_mut().zip(field_sizes.iter()) {
            field.offset = field_offset;
            field_offset += size;
        }
    }
}

fn add_padding(schema: &mut Schema) {
    let mut aligned_offsets = Vec::with_capacity(
        schema
            .types
            .iter()
            .map(|data_type| data_type.fields.len())
            .sum(),
    );

    for data_type in &schema.types {
        let mut offset = 0;
        let mut previous_cache_line = 0;
        for field in &data_type.fields {
            let size = field.size(schema);
            let absolute_aligned_offset = offset + (size - (offset % size)) % size;
            let current_cache_line = offset / 64;
            let cache_aligned_offset = if current_cache_line > previous_cache_line {
                offset
            } else {
                absolute_aligned_offset
            };

            offset = absolute_aligned_offset + size;
            previous_cache_line = current_cache_line;

            aligned_offsets.push(cache_aligned_offset);
        }
    }

    for (field, offset) in schema
        .types
        .iter_mut()
        .flat_map(|data_type| data_type.fields.iter_mut())
        .zip(aligned_offsets)
    {
        field.padding_size = offset - field.offset;
        field.offset = offset;
    }

    schema.is_packed = false;
}

fn optimize_schema(schema: &mut Schema, optimizations: SchemaOptimizations) {
    if optimizations.data_type_layout {
        optimize_data_type_layouts(schema);
    }
    if optimizations.add_padding {
        add_padding(schema);
    }
}

fn generate_schema(
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

fn generate_lang_files(path: &Path, gen_name: &str, schema: &Schema) -> Result<(), String> {
    match gen_name {
        "c" => {
            let mut generator = CGenerator::default();
            generator.feed(schema)?;
            generator.write(path)?;

            Ok(())
        }
        _ => Err(format!("Unknown language generator name: {gen_name}")),
    }
}

fn language_generator(path: &Path, gen_name: &str, schema: &Schema) -> Result<(), String> {
    generate_lang_files(path, gen_name, schema)
        .map_err(|err| format!("Failed to generate language files! Error: {err}"))
}

fn process_text(path: &Path, text: &str) -> Result<Vec<String>, String> {
    let mut status = vec![];

    let schema = generate_schema(Some(path.into()), text, SchemaOptimizations::new())?;
    let schema_serialized = serde_json::to_string(&schema);

    match schema_serialized {
        Ok(data) => {
            let output_path = path.with_extension("json");
            fs::write(&output_path, data).map_err(|err| {
                format!("Failed to write serialized schema to {output_path:?}! Error: {err:?}")
            })?;
            status.push(format!("Schema written to {output_path:?}"));
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
        status.push(format!(
            "Generated language files using {gen_name} generator."
        ));
    }

    Ok(status)
}

fn main() {
    for path in env::args()
        .skip(1)
        .map(PathBuf::from)
        .flat_map(fs::canonicalize)
    {
        println!("{path:?}");

        match fs::read_to_string(&path) {
            Ok(file_text) => match process_text(&path, &file_text) {
                Ok(status) => {
                    for line in status {
                        eprintln!("\t{line}")
                    }
                }
                Err(err) => eprintln!("\t{err}"),
            },
            Err(err) => eprintln!("{err:?}"),
        }

        println!()
    }
}
