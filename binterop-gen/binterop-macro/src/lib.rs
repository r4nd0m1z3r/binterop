#![feature(iter_intersperse)]
#![feature(proc_macro_span)]
#![feature(vec_into_raw_parts)]

mod binterop_derive;
mod binterop_inline;

#[proc_macro]
pub fn binterop_inline(token_stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    binterop_inline::binterop_inline(token_stream)
}

#[proc_macro_derive(Binterop)]
pub fn binterop_derive(token_stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    binterop_derive::derive_binterop(token_stream)
}
