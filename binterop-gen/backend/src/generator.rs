use crate::tokenizer::Token;
use crate::GENERATOR_DEBUG;
use binterop::field::Field;
use binterop::schema::Schema;
use binterop::types::array::ArrayType;
use binterop::types::data::DataType;
use binterop::types::function::{Arg, FunctionType};
use binterop::types::pointer::PointerType;
use binterop::types::primitives::INTEGER_PRIMITIVE_NAMES;
use binterop::types::r#enum::EnumType;
use binterop::types::union::UnionType;
use binterop::types::vector::VectorType;
use binterop::types::{Type, TypeData};

#[derive(Debug)]
pub struct Generator {
    currently_defining: Option<Type>,
    should_create_type: bool,
    next_is_repr_type: bool,
    next_is_fn_return_type: bool,
    current_index: usize,
    current_offset: usize,
    schema: Schema,
}
impl Default for Generator {
    fn default() -> Self {
        Self {
            currently_defining: None,
            should_create_type: false,
            next_is_repr_type: false,
            next_is_fn_return_type: false,
            current_index: 0,
            current_offset: 0,
            schema: Schema {
                is_packed: true,
                ..Default::default()
            },
        }
    }
}
impl Generator {
    fn process_ident(&mut self, ident: &str) -> Result<(), String> {
        if self.currently_defining.is_none() {
            return Err("Got ident while not defining anything!".to_string());
        }

        if self.should_create_type {
            match self.currently_defining.as_ref().unwrap() {
                Type::Primitive | Type::Array | Type::Vector | Type::String | Type::Pointer => {}
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
                Type::Function => {
                    self.current_index = self.schema.functions.len();
                    self.schema
                        .functions
                        .push(FunctionType::default_with_name(ident))
                }
            }

            self.should_create_type = false;
        } else {
            match self.currently_defining.as_ref().unwrap() {
                Type::Primitive | Type::Array | Type::Vector | Type::String | Type::Pointer => {}
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
                Type::Function => {
                    let current_fn = &mut self.schema.functions[self.current_index];

                    current_fn.args.push(Arg::new(ident.to_string(), None));
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
            ..
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

    fn process_vector(&mut self, name: &str) -> Result<usize, String> {
        let TypeData { index, r#type, .. } =
            self.schema.type_data_by_name(&name[1..name.len() - 1])?;

        let ptr_type = PointerType::new(r#type, index);
        self.schema.pointers.push(ptr_type);

        let new_field = self.schema.types[self.current_index]
            .fields
            .last_mut()
            .unwrap();
        new_field.r#type = Type::Vector;
        new_field.type_index = self.schema.vectors.len();
        new_field.offset = self.current_offset;

        let vector_type = VectorType::new(r#type, index);
        self.schema.vectors.push(vector_type);

        Ok(VectorType::size())
    }

    fn process_field(&mut self, name: &str) -> Result<usize, String> {
        let TypeData {
            index,
            r#type,
            size,
            ..
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

    fn process_function(&mut self, name: &str) -> Result<usize, String> {
        let r#type = self.schema.type_data_by_name(name)?;

        if self.next_is_fn_return_type {
            self.next_is_fn_return_type = false;
            self.currently_defining = None;

            self.schema.functions[self.current_index].return_type = Some(r#type);
        } else {
            let arg_type_data = self.schema.type_data_by_name(name)?;
            let current_fn = &mut self.schema.functions[self.current_index];

            if let Some(arg) = current_fn.args.last_mut() {
                arg.r#type = Some(arg_type_data);
            }
        }

        Ok(FunctionType::size())
    }

    fn process_type(&mut self, name: &str) -> Result<(), String> {
        if (self.currently_defining == Some(Type::Enum)
            || self.currently_defining == Some(Type::Union))
            && !INTEGER_PRIMITIVE_NAMES.contains(&name)
        {
            return Err(format!("{name:?} cannot represent enum state since it was not found in integer primitive list!\n\tAvailable integer primitives: {INTEGER_PRIMITIVE_NAMES:?}"));
        }

        let size = if self.currently_defining == Some(Type::Function) {
            self.process_function(name)?
        } else if name.ends_with('*') {
            self.process_pointer(name)?
        } else if name.starts_with('[') && name.ends_with(']') {
            self.process_array(name)?
        } else if name.starts_with('<') && name.ends_with('>') {
            self.process_vector(name)?
        } else {
            let currently_defined_type = &self.schema.types[self.current_index];

            if name == currently_defined_type.name {
                currently_defined_type
                    .fields
                    .iter()
                    .map(|field| field.size(&self.schema))
                    .sum()
            } else {
                self.process_field(name)?
            }
        };

        self.current_offset += size;

        Ok(())
    }

    pub fn feed(&mut self, token: Token) -> Result<(), String> {
        if *GENERATOR_DEBUG {
            eprintln!("{token:?}");
        }

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
            Token::Fn => {
                self.currently_defining = Some(Type::Function);
                self.should_create_type = true;
            }
            Token::FnReturn => {
                self.next_is_fn_return_type = true;
            }
        }

        Ok(())
    }

    pub fn output(&mut self) -> Schema {
        self.schema.clone()
    }
}
