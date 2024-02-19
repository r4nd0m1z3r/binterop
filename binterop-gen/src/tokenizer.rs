use std::borrow::{Borrow, Cow};
use std::collections::VecDeque;
use std::fs;

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

pub struct Tokenizer<'a> {
    current_include_chunks: Option<VecDeque<Cow<'a, str>>>,
    text_chunks: VecDeque<Cow<'a, str>>,
    next_is_type: bool,
}
impl<'a> Tokenizer<'a> {
    fn prepare_text(text: &str) -> VecDeque<Cow<'a, str>> {
        text.replace(['\n', ','], " ")
            .split(' ')
            .filter(|chunk| !chunk.is_empty())
            .map(|chunk| Cow::from(chunk.to_string()))
            .collect()
    }

    fn match_chunk(&mut self, chunk: Cow<'a, str>) -> Option<Token> {
        match chunk.borrow() {
            "include" => {
                let chunk_source = if let Some(chunks) = self.current_include_chunks.as_mut() {
                    chunks
                } else {
                    &mut self.text_chunks
                };

                let include_text = fs::read_to_string(chunk_source.pop_front()?.as_ref()).ok()?;
                let mut include_chunks = Self::prepare_text(&include_text);
                let this_chunk = include_chunks.pop_front()?;

                self.current_include_chunks = Some(include_chunks);

                self.match_chunk(this_chunk)
            }
            "root" => Some(Token::Root),
            "{" => Some(Token::DefBegin),
            "}" => {
                if self.current_include_chunks.is_some() {
                    self.current_include_chunks = None
                }
                Some(Token::DefEnd)
            }
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
                            .map(|chunk| Cow::from(chunk.to_owned()))?,
                    ))
                }
            }
        }
    }

    pub fn new(text: &str) -> Self {
        Self {
            current_include_chunks: None,
            text_chunks: Self::prepare_text(text),
            next_is_type: false,
        }
    }

    pub fn yield_token(&mut self) -> Option<Token> {
        if self.text_chunks.is_empty() {
            return None;
        }

        let chunk_source = if self.current_include_chunks.is_some() {
            self.current_include_chunks.as_mut()?
        } else {
            &mut self.text_chunks
        };

        let chunk = chunk_source.pop_front()?;
        self.match_chunk(chunk)
    }
}
