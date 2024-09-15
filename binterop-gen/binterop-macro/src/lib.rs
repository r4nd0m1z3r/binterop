#![feature(iter_intersperse)]
#![feature(proc_macro_span)]
#![feature(vec_into_raw_parts)]

use backend::helpers::{generate_schema, serialize_schema};
use backend::language_generators::rust_gen::RustGenerator;
use backend::language_generators::LanguageGenerator;
use backend::optimization::SchemaOptimizations;
use proc_macro::TokenTree;
use proc_macro2::TokenStream;
use quote::quote;
use std::path::PathBuf;
use std::{env, fs};
use syn::{Data, DataEnum, DataStruct, DataUnion, DeriveInput};

#[proc_macro]
pub fn binterop_inline(token_stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
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

fn struct_derive(data_struct: DataStruct) -> TokenStream {
    let fields_tokens = data_struct
        .fields
        .iter()
        .map(|field| {
            let type_name = &field.ty;
            let field = quote! {
                Field::new_from_wrapped(&(#type_name::binterop_type(schema)), schema),
            };

            Into::<TokenStream>::into(field)
        })
        .collect::<TokenStream>();

    quote! {
        fn binterop_type(schema: &mut Schema) -> WrappedType {
            let mut data_type = DataType::from_fields(
                type_name::<Self>(),
                &[#fields_tokens]
            );

            let mut layout = Layout::from_size_align(0, 1).unwrap();
            for field in &mut data_type.fields {
                let field_layout = field.layout(schema);
                let (new_layout, offset) = layout.extend(field_layout).unwrap();
                layout = new_layout;
                field.offset = offset;
            }

            WrappedType::Data(data_type)
        }
    }
}

fn enum_derive(data_enum: DataEnum) -> TokenStream {
    unimplemented!()
}

fn union_derive(data_union: DataUnion) -> TokenStream {
    unimplemented!()
}

#[proc_macro_derive(Binterop)]
pub fn derive_binterop(token_stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse(token_stream).unwrap();

    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let binterop_impl = match input.data {
        Data::Struct(data_struct) => struct_derive(data_struct),
        Data::Enum(data_enum) => enum_derive(data_enum),
        Data::Union(data_union) => union_derive(data_union),
    };

    let expanded = quote! {
        use binterop::types::{WrappedType, data::DataType};
        use binterop::{Binterop, schema::Schema, std::Vector};
        use binterop::field::Field;
        use std::alloc::Layout;
        use std::any::type_name;

        impl #impl_generics binterop::Binterop for #name #ty_generics #where_clause {
            #binterop_impl
        }
    };

    expanded.into()
}
