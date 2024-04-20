#![feature(iter_intersperse)]

use backend::helpers::process_text;
use proc_macro::{TokenStream, TokenTree};
use std::fs;
use std::panic::{set_hook, PanicInfo};

fn panic_handler(_: &PanicInfo) {
    fs::remove_dir_all("binterop_generated")
        .unwrap_or_else(|err| eprintln!("Failed to remove generated files folder! Error: {err:?}"));
}

#[proc_macro]
pub fn binterop_inline(token_stream: TokenStream) -> TokenStream {
    set_hook(Box::new(panic_handler));

    let mut token_stream_iter = token_stream.into_iter();

    let name = token_stream_iter
        .by_ref()
        .take_while(|token| token.to_string() != ",")
        .map(|token| {
            if let TokenTree::Literal(literal) = token {
                literal.to_string()
            } else {
                String::new()
            }
        })
        .collect::<String>();
    assert!(!name.is_empty(), "Specify name for generated schema file!");

    let schema_text = token_stream_iter
        .map(|tree| tree.to_string())
        .intersperse(" ".to_string())
        .collect::<String>();

    fs::remove_dir_all("binterop_generated")
        .unwrap_or_else(|err| eprintln!("Failed to remove generated files folder! Error: {err:?}"));
    fs::create_dir_all("binterop_generated").unwrap();
    process_text(
        format!("binterop_generated/{}", name.trim_matches('"')).as_ref(),
        &schema_text,
    )
    .unwrap();

    TokenStream::new()
}
