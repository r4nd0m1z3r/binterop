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
    include_chunks_queue: Vec<VecDeque<Cow<'a, str>>>,
    text_chunks: VecDeque<Cow<'a, str>>,
    next_is_type: bool,
}
impl<'a> Tokenizer<'a> {
    fn prepare_text(text: &str) -> VecDeque<Cow<'a, str>> {
        text.replace(['\n', '\r', ','], " ")
            .split(' ')
            .filter(|chunk| !chunk.is_empty())
            .map(|chunk| Cow::from(chunk.to_string()))
            .collect()
    }

    fn match_chunk(&mut self, chunk: Cow<'a, str>) -> Result<Option<Token>, String> {
        match chunk.borrow() {
            "include" => {
                let chunk_source = if let Some(chunks) = self.include_chunks_queue.last_mut() {
                    chunks
                } else {
                    &mut self.text_chunks
                };

                let path = chunk_source.pop_front().ok_or("No path token after include was found!")?;
                let include_text = fs::read_to_string(path.as_ref()).map_err(|err| err.to_string())?;
                let mut include_chunks = Self::prepare_text(&include_text);
                let this_chunk = include_chunks.pop_front().ok_or(format!("Include file {path:?} contains no tokens!"))?;

                self.include_chunks_queue.push(include_chunks);

                self.match_chunk(this_chunk)
            }
            "root" => Ok(Some(Token::Root)),
            "{" => Ok(Some(Token::DefBegin)),
            "}" => Ok(Some(Token::DefEnd)),
            "struct" => Ok(Some(Token::Struct)),
            "enum" => Ok(Some(Token::Enum)),
            _ => {
                if chunk.chars().all(char::is_alphanumeric) {
                    if self.next_is_type {
                        self.next_is_type = false;
                        Ok(Some(Token::Type(chunk)))
                    } else {
                        Ok(Some(Token::Ident(chunk)))
                    }
                } else {
                    self.next_is_type = chunk.ends_with(':');
                    Ok(Some(Token::Ident(
                        chunk
                            .strip_suffix(':')
                            .map(|chunk| Cow::from(chunk.to_owned())).ok_or(format!("Expected field ident but got {chunk:?}"))?,
                    )))
                }
            }
        }
    }

    pub fn new(text: &str) -> Self {
        Self {
            include_chunks_queue: vec![],
            text_chunks: Self::prepare_text(text),
            next_is_type: false,
        }
    }

    pub fn yield_token(&mut self) -> Result<Option<Token>, String> {
        if self.text_chunks.is_empty() {
            return Ok(None);
        }

        if let Some(chunks) = self.include_chunks_queue.last() {
            if chunks.is_empty() {
                self.include_chunks_queue.pop();
            }
        }

        let chunk_source = if !self.include_chunks_queue.is_empty() {
            self.include_chunks_queue.last_mut().unwrap()
        } else {
            &mut self.text_chunks
        };

        let chunk = chunk_source.pop_front().unwrap();
        self.match_chunk(chunk)
    }
}
