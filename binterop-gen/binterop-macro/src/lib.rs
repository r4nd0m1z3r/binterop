#![feature(iter_intersperse)]
#![feature(proc_macro_span)]

use backend::helpers::{generate_schema, serialize_schema};
use backend::language_generators::rust_gen::RustGenerator;
use backend::language_generators::LanguageGenerator;
use backend::optimization::SchemaOptimizations;
use proc_macro::{TokenStream, TokenTree};
use std::panic::{set_hook, PanicInfo};
use std::path::PathBuf;
use std::{env, fs};

fn panic_handler(_: &PanicInfo) {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .map(|mut dir| {
            dir.push('/');
            dir
        })
        .unwrap_or_default();
    let generated_dir = PathBuf::from(format!("{manifest_dir}binterop_generated"));

    fs::remove_dir_all(generated_dir)
        .unwrap_or_else(|err| eprintln!("Failed to remove generated files folder! Error: {err:?}"));
}

#[proc_macro]
pub fn binterop_inline(token_stream: TokenStream) -> TokenStream {
    set_hook(Box::new(panic_handler));

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
    let schema = generate_schema(None, &schema_text, SchemaOptimizations::default()).unwrap();

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
            .with_extension("rs"),
        serialize_schema(&schema).unwrap(),
    )
    .unwrap();

    let mut generator = RustGenerator::default();

    let helpers_path = file_path.with_file_name("helpers").with_extension("rs");
    if helpers_path.try_exists().is_ok() {
        generator.output.clear();
    }

    generator.feed(&schema).unwrap();

    fs::write(&helpers_path, &generator.helpers_output).unwrap();
    generator.output.parse().unwrap()
}
