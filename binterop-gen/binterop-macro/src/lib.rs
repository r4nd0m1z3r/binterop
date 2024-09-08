#![feature(iter_intersperse)]
#![feature(proc_macro_span)]

use backend::helpers::{generate_schema, serialize_schema};
use backend::language_generators::rust_gen::RustGenerator;
use backend::language_generators::LanguageGenerator;
use backend::optimization::SchemaOptimizations;
use proc_macro::{TokenStream, TokenTree};
use quote::quote;
use std::path::PathBuf;
use std::{env, fs};
use syn::{Data, DataEnum, DataStruct, DataUnion, DeriveInput};

#[proc_macro]
pub fn binterop_inline(token_stream: TokenStream) -> TokenStream {
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
        return TokenStream::new();
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

fn struct_derive(data_struct: DataStruct) -> TokenStream {
    data_struct
        .fields
        .into_iter()
        .map(|field| {
            let field_name = field.ident;
            let quote = quote! {};

            Into::<TokenStream>::into(quote)
        })
        .collect()
}

fn enum_derive(data_enum: DataEnum) -> TokenStream {
    unimplemented!()
}

fn union_derive(data_union: DataUnion) -> TokenStream {
    unimplemented!()
}

#[proc_macro_derive(Binterop)]
pub fn derive_binterop(token_stream: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(token_stream).unwrap();

    match input.data {
        Data::Struct(data_struct) => {
            let out = struct_derive(data_struct);
            dbg!(out.to_string());
            out
        }
        Data::Enum(data_enum) => enum_derive(data_enum),
        Data::Union(data_union) => union_derive(data_union),
    }
}
