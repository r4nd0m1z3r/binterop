use binterop::field::Field;
use binterop::primitives::PRIMITIVES;
use binterop::schema::{Schema, Type};
use binterop::types::{DataType, EnumType};
use std::borrow::Borrow;
use std::borrow::Cow;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::{env, fs};

#[derive(Clone, Debug)]
enum Token<'a> {
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

#[derive(Debug, Default)]
struct Generator {
    currently_defining: Option<Type>,
    should_create_type: bool,
    root_index: usize,
    current_index: usize,
    current_offset: usize,
    schema: Schema,
}
impl Generator {
    fn feed(&mut self, token: Token) {
        dbg!(&token);

        match token {
            Token::Ident(ident) => {
                assert!(
                    self.currently_defining.is_some(),
                    "Got ident while not defining anything!"
                );

                if self.should_create_type {
                    match self.currently_defining.as_ref().unwrap() {
                        Type::Data => {
                            self.current_index = self.schema.types.len();
                            self.schema.types.push(DataType::default_with_name(&ident))
                        }
                        Type::Enum => {
                            self.current_index = self.schema.enums.len();
                            self.schema.enums.push(EnumType::default_with_name(&ident))
                        }
                        _ => {}
                    }
                    self.should_create_type = false;
                } else {
                    match self.currently_defining.as_ref().unwrap() {
                        Type::Data => self.schema.types[self.current_index]
                            .fields
                            .push(Field::default_with_name(&ident)),
                        Type::Enum => self.schema.enums[self.current_index]
                            .variants
                            .push(ident.to_string()),
                        _ => {}
                    }
                }
            }
            Token::Root => self.root_index = self.current_index,
            Token::DefBegin => {}
            Token::DefEnd => {
                self.currently_defining = None;
            }
            Token::Struct => {
                self.currently_defining = Some(Type::Data);
                self.should_create_type = true;
            }
            Token::Enum => {
                self.currently_defining = Some(Type::Enum);
                self.should_create_type = true;
            }
            Token::Type(name) => {
                let (type_index, r#type, type_size) = (|| {
                    if let Some(index) = PRIMITIVES
                        .into_iter()
                        .enumerate()
                        .find(|(_, (_, primitive))| primitive.name == name)
                        .map(|(index, (_, _))| index)
                    {
                        let type_size = PRIMITIVES.index(index).map(|(_, v)| v.size).unwrap();
                        return Some((index, Type::Primitive, type_size));
                    }

                    if let Some(index) = self
                        .schema
                        .types
                        .iter()
                        .enumerate()
                        .find(|(_, data_type)| data_type.name == name)
                        .map(|(index, _)| index)
                    {
                        let type_size = self.schema.types[index].size(&self.schema);
                        return Some((index, Type::Data, type_size));
                    }

                    if let Some(index) = self
                        .schema
                        .enums
                        .iter()
                        .enumerate()
                        .find(|(_, enum_type)| enum_type.name == *name)
                        .map(|(index, _)| index)
                    {
                        let type_size = self.schema.enums[index].size();
                        return Some((index, Type::Enum, type_size));
                    }
                    None
                })()
                .unwrap_or_else(|| panic!("Failed to find type with name {name:?}"));

                dbg!(&self.schema.types[self.current_index]);
                let new_field = self.schema.types[self.current_index]
                    .fields
                    .last_mut()
                    .unwrap();

                new_field.r#type = r#type;
                new_field.type_index = type_index;
                new_field.offset = self.current_offset;

                self.current_offset += type_size;
            }
        }
    }

    fn get_schema(&mut self) -> Schema {
        self.schema.root_type_index = self.root_index;
        self.schema.clone()
    }
}

fn generate_schema(definition_text: &str) -> Schema {
    let mut tokenizer = Tokenizer::new(definition_text);
    let mut generator = Generator::default();

    while let Some(token) = tokenizer.yield_token() {
        generator.feed(token);
    }

    generator.get_schema()
}

fn main() {
    let mut args_iter = env::args();
    args_iter.next();

    for path in args_iter.map(PathBuf::from) {
        match fs::read(&path) {
            Ok(data) => {
                let definition_text = String::from_utf8_lossy(&data);
                let schema = generate_schema(&definition_text);
                let schema_serialized = serde_json::to_string(&schema);

                if let Ok(data) = schema_serialized {
                    let output_path = path.with_extension("json");
                    if let Err(err) = fs::write(&output_path, data) {
                        eprintln!("Failed to write serialized schema for {path:?}! Error: {err:?}");
                    } else {
                        println!("{path:?} -> {output_path:?}");
                    }
                } else {
                    eprintln!("Failed to serialize {path:?} schema!");
                }
            }
            Err(err) => eprintln!("Failed to load file from {path:?}! Error: {err:?}"),
        }
    }
}
