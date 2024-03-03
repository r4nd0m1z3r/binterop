use crate::tokenizer::Token;
use binterop::field::Field;
use binterop::primitives::INTEGER_PRIMITIVE_NAMES;
use binterop::schema::{Schema, Type};
use binterop::types::{DataType, EnumType, UnionType};

#[derive(Debug, Default)]
pub struct Generator {
    currently_defining: Option<Type>,
    should_create_type: bool,
    next_is_repr_type: bool,
    root_index: usize,
    current_index: usize,
    current_offset: usize,
    schema: Schema,
}
impl Generator {
    fn process_ident(&mut self, ident: &str) -> Result<(), String> {
        if self.currently_defining.is_none() {
            return Err("Got ident while not defining anything!".to_string());
        }

        if self.should_create_type {
            match self.currently_defining.as_ref().unwrap() {
                Type::Primitive => {}
                Type::Data => {
                    self.current_index = self.schema.types.len();
                    self.schema.types.push(DataType::default_with_name(ident))
                }
                Type::Enum => {
                    self.current_index = self.schema.enums.len();
                    self.schema.enums.push(EnumType::default_with_name(ident))
                }
                Type::Union => {
                    self.current_index = self.schema.unions.len();
                    self.schema.unions.push(UnionType::default_with_name(ident))
                }
            }

            self.should_create_type = false;
        } else {
            match self.currently_defining.as_ref().unwrap() {
                Type::Primitive => {}
                Type::Data => self.schema.types[self.current_index]
                    .fields
                    .push(Field::default_with_name(ident)),
                Type::Enum => self.schema.enums[self.current_index]
                    .variants
                    .push(ident.to_string()),
                Type::Union => {
                    let r#type = self
                        .schema
                        .type_data_by_name(ident)
                        .map(|(index, r#type, _)| (index, r#type))?;
                    self.schema.unions[self.current_index]
                        .possible_types
                        .push(r#type)
                }
            }
        }

        Ok(())
    }

    fn process_type(&mut self, name: &str) -> Result<(), String> {
        if (self.currently_defining == Some(Type::Enum)
            || self.currently_defining == Some(Type::Union))
            && !INTEGER_PRIMITIVE_NAMES.contains(&name)
        {
            return Err(format!("{name:?} cannot represent enum state since it was not found in integer primitive list!\n\tAvailable integer primitives: {INTEGER_PRIMITIVE_NAMES:?}"));
        }

        let (type_index, r#type, type_size) = self.schema.type_data_by_name(name)?;

        match self.currently_defining {
            Some(Type::Enum) => {
                self.schema.enums[self.current_index].repr_type_index = type_index;
            }
            Some(Type::Union) => {
                self.schema.unions[self.current_index].repr_type_index = type_index;
            }
            _ => {
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

        Ok(())
    }

    pub(crate) fn feed(&mut self, token: Token) -> Result<(), String> {
        match token {
            Token::Ident(ident) => {
                if self.next_is_repr_type {
                    self.feed(Token::Type(ident.clone()))?;
                } else {
                    self.process_ident(&ident)?;
                }

                self.next_is_repr_type = match self.currently_defining {
                    Some(Type::Enum) => {
                        !self.should_create_type
                            && self.schema.enums[self.current_index].variants.is_empty()
                    }
                    Some(Type::Union) => {
                        !self.should_create_type
                            && self.schema.unions[self.current_index]
                                .possible_types
                                .is_empty()
                    }
                    _ => false,
                };
            }
            Token::Root => self.root_index = self.current_index,
            Token::DefBegin => {
                self.next_is_repr_type = false;
            }
            Token::DefEnd => {
                self.currently_defining = None;
                self.current_offset = 0;
            }
            Token::Struct => {
                self.currently_defining = Some(Type::Data);
                self.should_create_type = true;
            }
            Token::Enum => {
                self.currently_defining = Some(Type::Enum);
                self.should_create_type = true;
            }
            Token::Union => {
                self.currently_defining = Some(Type::Union);
                self.should_create_type = true;
            }
            Token::Type(name) => self.process_type(&name)?,
        }

        Ok(())
    }

    pub(crate) fn output(&mut self) -> Schema {
        self.schema.root_type_index = self.root_index;
        self.schema.clone()
    }
}
