use crate::tokenizer::Token;
use binterop::field::Field;
use binterop::primitives::PRIMITIVES;
use binterop::schema::{Schema, Type};
use binterop::types::{DataType, EnumType};

#[derive(Debug, Default)]
pub struct Generator {
    currently_defining: Option<Type>,
    should_create_type: bool,
    root_index: usize,
    current_index: usize,
    current_offset: usize,
    schema: Schema,
}
impl Generator {
    pub(crate) fn feed(&mut self, token: Token) {
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
                .unwrap_or_else(|| {
                    let available_type_names = self.schema.types.iter().map(|data_type| data_type.name.clone()).collect::<Vec<_>>();
                    let available_enum_names = self.schema.enums.iter().map(|enum_type| enum_type.name.clone()).collect::<Vec<_>>();
                    panic!("Failed to find type with name {name:?}!\n\tAvailable types: {:?}\n\tAvailable enums: {:?}", available_type_names, available_enum_names)
                });

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

    pub(crate) fn get_schema(&mut self) -> Schema {
        self.schema.root_type_index = self.root_index;
        self.schema.clone()
    }
}
