use std::borrow::{Borrow, Cow};
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub enum Token<'a> {
    Ident(Cow<'a, str>),
    Root,
    DefBegin,
    DefEnd,
    Struct,
    Enum,
    Type(Cow<'a, str>),
}

struct Tokenizer<'a> {
    text_chunks: VecDeque<Cow<'a, str>>,
    next_is_type: bool,
}
impl<'a> Tokenizer<'a> {
    pub fn new(text: &str) -> Self {
        let text_chunks = text
            .replace(['\n', ','], " ")
            .split(' ')
            .filter(|chunk| !chunk.is_empty())
            .map(|chunk| Cow::from(chunk.to_string()))
            .collect();

        Self {
            text_chunks,
            next_is_type: false,
        }
    }

    pub fn yield_token(&mut self) -> Option<Token> {
        if self.text_chunks.is_empty() {
            return None;
        }

        let chunk = self.text_chunks.pop_front()?;

        match chunk.borrow() {
            "root" => Some(Token::Root),
            "{" => Some(Token::DefBegin),
            "}" => Some(Token::DefEnd),
            "struct" => Some(Token::Struct),
            "enum" => Some(Token::Enum),
            _ => {
                if chunk.chars().all(char::is_alphanumeric) {
                    if self.next_is_type {
                        self.next_is_type = false;
                        Some(Token::Type(chunk))
                    } else {
                        Some(Token::Ident(chunk))
                    }
                } else {
                    self.next_is_type = chunk.ends_with(':');
                    Some(Token::Ident(
                        chunk
                            .strip_suffix(':')
                            .map(|chunk| Cow::from(chunk.to_owned()))
                            .unwrap(),
                    ))
                }
            }
        }
    }
}
