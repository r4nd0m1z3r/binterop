use crate::tokenizer::Token;
use binterop::field::Field;
use binterop::schema::Schema;
use binterop::types::array::ArrayType;
use binterop::types::data::DataType;
use binterop::types::heap_array::HeapArrayType;
use binterop::types::pointer::PointerType;
use binterop::types::primitives::INTEGER_PRIMITIVE_NAMES;
use binterop::types::r#enum::EnumType;
use binterop::types::union::UnionType;
use binterop::types::{Type, TypeData};

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
                Type::Primitive | Type::Array | Type::HeapArray | Type::Pointer => {}
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
                Type::Primitive | Type::Array | Type::HeapArray | Type::Pointer => {}
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
                        .map(|type_data| (type_data.index, type_data.r#type))?;
                    self.schema.unions[self.current_index]
                        .possible_types
                        .push(r#type)
                }
            }
        }

        Ok(())
    }

    fn process_pointer(&mut self, name: &str) -> Result<usize, String> {
        let TypeData { index, r#type, .. } = self
            .schema
            .type_data_by_name(name.strip_suffix('*').unwrap())?;
        let pointer_type = PointerType::new(r#type, index);

        let new_field = self.schema.types[self.current_index]
            .fields
            .last_mut()
            .unwrap();
        new_field.r#type = Type::Pointer;
        new_field.type_index = self.schema.pointers.len();
        new_field.offset = self.current_offset;

        self.schema.pointers.push(pointer_type);

        Ok(PointerType::size())
    }

    fn process_array(&mut self, name: &str) -> Result<usize, String> {
        let separator_index = name
            .chars()
            .position(|ch| ch == ':')
            .ok_or("Expected array type but failed to find separator!")?;

        let TypeData {
            index,
            r#type,
            size: inner_type_size,
        } = self.schema.type_data_by_name(&name[1..separator_index])?;

        let array_len = name[separator_index + 1..name.len() - 1]
            .parse::<usize>()
            .map_err(|err| format!("Failed to parse array len! Error: {err:?}"))?;

        let array_type = ArrayType::new(r#type, index, array_len);

        let new_field = self.schema.types[self.current_index]
            .fields
            .last_mut()
            .unwrap();
        new_field.r#type = Type::Array;
        new_field.type_index = self.schema.arrays.len();
        new_field.offset = self.current_offset;

        self.schema.arrays.push(array_type);

        Ok(inner_type_size)
    }

    fn process_heap_array(&mut self, name: &str) -> Result<usize, String> {
        let TypeData { index, r#type, .. } =
            self.schema.type_data_by_name(&name[1..name.len() - 1])?;

        let ptr_type = PointerType::new(r#type, index);
        self.schema.pointers.push(ptr_type);

        let new_field = self.schema.types[self.current_index]
            .fields
            .last_mut()
            .unwrap();
        new_field.r#type = Type::HeapArray;
        new_field.type_index = self.schema.heap_arrays.len();
        new_field.offset = self.current_offset;

        let heap_array_type = HeapArrayType::new(r#type, index);
        self.schema.heap_arrays.push(heap_array_type);

        Ok(HeapArrayType::size())
    }

    fn process_field(&mut self, name: &str) -> Result<usize, String> {
        let TypeData {
            index,
            r#type,
            size,
        } = self.schema.type_data_by_name(name)?;

        let new_field = self.schema.types[self.current_index]
            .fields
            .last_mut()
            .unwrap();
        new_field.r#type = r#type;
        new_field.type_index = index;
        new_field.offset = self.current_offset;

        Ok(size)
    }

    fn process_type(&mut self, name: &str) -> Result<(), String> {
        if (self.currently_defining == Some(Type::Enum)
            || self.currently_defining == Some(Type::Union))
            && !INTEGER_PRIMITIVE_NAMES.contains(&name)
        {
            return Err(format!("{name:?} cannot represent enum state since it was not found in integer primitive list!\n\tAvailable integer primitives: {INTEGER_PRIMITIVE_NAMES:?}"));
        }

        match self.currently_defining {
            Some(Type::Enum) => {}
            Some(Type::Union) => {
                self.schema.unions[self.current_index].repr_type_index =
                    self.schema.type_data_by_name(name)?.index;
            }
            _ => {
                let size = if name.ends_with('*') {
                    self.process_pointer(name)?
                } else if name.starts_with('[') && name.ends_with(']') {
                    self.process_array(name)?
                } else if name.starts_with('<') && name.ends_with('>') {
                    self.process_heap_array(name)?
                } else {
                    self.process_field(name)?
                };

                self.current_offset += size;
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
            Token::Root => self.root_index = self.schema.types.len(),
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
