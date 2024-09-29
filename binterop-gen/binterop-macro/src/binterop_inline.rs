use std::{env, fs, path::PathBuf};

use backend::{
    helpers::{generate_schema, serialize_schema},
    language_generators::{rust_gen::RustGenerator, LanguageGenerator},
    optimization::SchemaOptimizations,
};
use proc_macro::TokenTree;
use proc_macro2::TokenStream;

pub(crate) fn binterop_inline(token_stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut token_stream_iter = token_stream.into_iter();

    let mut name = String::new();
    let mut file_path = PathBuf::new();
    for token in token_stream_iter
        .by_ref()
        .take_while(|token| token.to_string() != ",")
    {
        if let TokenTree::Literal(literal) = token {
            name = literal.to_string();
            file_path = literal.span().source_file().path();
        }
    }

    assert!(!name.is_empty(), "Specify name for generated schema file!");

    let schema_text = token_stream_iter
        .map(|tree| tree.to_string())
        .intersperse(" ".to_string())
        .collect::<String>();
    if schema_text.is_empty() {
        return TokenStream::new().into();
    }

    let schema = generate_schema(
        Some(file_path.clone()),
        &schema_text,
        SchemaOptimizations::default(),
    )
    .unwrap();

    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .map(|mut dir| {
            dir.push('/');
            dir
        })
        .unwrap_or_default();
    let generated_dir = PathBuf::from(format!("{manifest_dir}binterop_generated/"));

    fs::create_dir_all(&generated_dir).unwrap();

    fs::write(
        generated_dir
            .join(name.trim_matches('"'))
            .with_extension("bintdef"),
        schema_text,
    )
    .unwrap();
    fs::write(
        generated_dir
            .join(name.trim_matches('"'))
            .with_extension("json"),
        serialize_schema(&schema).unwrap(),
    )
    .unwrap();

    let mut generator = RustGenerator::default();
    generator.output.clear();

    let helpers_path = file_path.with_file_name("helpers").with_extension("rs");

    generator.feed(&schema).unwrap();

    fs::write(helpers_path, &generator.helpers_output).unwrap();
    generator.output.parse().unwrap()
}
