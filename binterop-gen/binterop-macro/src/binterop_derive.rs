use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Data, DataEnum, DataStruct, DataUnion, DeriveInput};

use crate::has_repr;

fn struct_derive(data_struct: DataStruct) -> TokenStream {
    let fields_tokens = data_struct
        .fields
        .iter()
        .map(|field| {
            let mut type_name = field.ty.to_token_stream().to_string();

            let insertion_indices = type_name
                .chars()
                .enumerate()
                .flat_map(|(index, ch)| if ch == '<' { Some(index) } else { None })
                .collect::<Vec<_>>();

            for index in insertion_indices {
                type_name.insert_str(index, "::");
            }

            if let (Some('['), Some(']')) = (type_name.chars().next(), type_name.chars().last()) {
                type_name = format!("<{type_name}>");
            }

            let type_name: TokenStream = syn::parse_str(&type_name).unwrap();
            let field = quote! {
                Field::new_from_wrapped(&(#type_name::binterop_type(schema)), schema),
            };

            Into::<TokenStream>::into(field)
        })
        .collect::<TokenStream>();

    quote! {
        fn binterop_type(schema: &mut binterop::schema::Schema) -> binterop::types::WrappedType {
            use binterop::types::{WrappedType, data::DataType};
            use binterop::field::Field;
            use std::alloc::Layout;
            use std::any::type_name;

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
    let variant_names = data_enum
        .variants
        .iter()
        .map(|variant| {
            let variant = variant.ident.to_string();
            quote!(#variant,)
        })
        .collect::<TokenStream>();

    quote! {
        fn binterop_type(schema: &mut binterop::schema::Schema) -> binterop::types::WrappedType {
            use binterop::types::{WrappedType, r#enum::EnumType};
            use binterop::field::Field;
            use std::alloc::Layout;
            use std::any::type_name;

            let enum_type = EnumType::new(type_name::<Self>(), &[#variant_names]);

            WrappedType::Enum(enum_type)
        }
    }
}

fn union_derive(data_union: DataUnion) -> TokenStream {
    unimplemented!()
}

pub(crate) fn derive_binterop(token_stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse(token_stream).unwrap();

    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    assert!(
        has_repr(&input.attrs, "C"),
        "Binterop can only work with C structure layout!"
    );

    let binterop_impl = match input.data {
        Data::Struct(data_struct) => struct_derive(data_struct),
        Data::Enum(data_enum) => enum_derive(data_enum),
        Data::Union(data_union) => union_derive(data_union),
    };

    let expanded = quote! {
        impl #impl_generics binterop::Binterop for #name #ty_generics #where_clause {
            #binterop_impl
        }
    };

    expanded.into()
}
