#![feature(iter_intersperse)]
#![feature(proc_macro_span)]
#![feature(vec_into_raw_parts)]

use syn::{AttrStyle, Attribute, Meta::List, MetaList, Path};

mod binterop_derive;
mod binterop_inline;

pub(crate) fn has_repr(attrs: &[Attribute], repr: &str) -> bool {
    for attr in attrs {
        // If the style isn't outer, reject it
        if !matches!(attr.style, AttrStyle::Outer) {
            continue;
        }

        // If the path doesn't match, reject it
        if let Path {
            leading_colon: None,
            ref segments,
        } = attr.path()
        {
            // If there's more than one, reject it
            if segments.len() != 1 {
                continue;
            }

            let seg = segments.first().unwrap();

            // If there are arguments, reject it
            if !seg.arguments.is_empty() {
                continue;
            }

            // If the ident isn't "repr", reject it
            if seg.ident != "repr" {
                continue;
            }
        } else {
            // If we don't match, reject if
            continue;
        }

        if let List(MetaList {
            path: _,
            delimiter: _,
            tokens,
        }) = &attr.meta
        {
            // If it doesn't match, reject it
            if tokens.to_string() != repr {
                continue;
            }
        }

        return true;
    }

    false
}

#[proc_macro]
pub fn binterop_inline(token_stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    binterop_inline::binterop_inline(token_stream)
}

#[proc_macro_derive(Binterop)]
pub fn binterop_derive(token_stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    binterop_derive::derive_binterop(token_stream)
}
