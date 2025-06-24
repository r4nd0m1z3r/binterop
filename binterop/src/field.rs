use crate::types::Type;
use crate::{schema::Schema, types::WrappedType};
use serde::{Deserialize, Serialize};
use std::{alloc::Layout, borrow::Cow};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub r#type: Type,
    pub type_index: usize,
    pub offset: usize,
    pub padding_size: usize,
}
impl Field {
    pub fn new(
        name: &str,
        r#type: Type,
        type_index: usize,
        offset: usize,
        padding_size: usize,
    ) -> Self {
        Self {
            name: name.to_string(),
            r#type,
            type_index,
            offset,
            padding_size,
        }
    }

    pub fn default_with_name(name: &str) -> Self {
        Self {
            name: name.to_string(),
            r#type: Type::Primitive,
            type_index: 0,
            offset: 0,
            padding_size: 0,
        }
    }

    pub fn new_from_wrapped(name: &str, wrapped_type: &WrappedType, schema: &Schema) -> Self {
        let type_index = schema.wrapped_type_index(wrapped_type).unwrap_or_else(|| {
            if let WrappedType::String = wrapped_type {
                0
            } else {
                panic!("Provided schema doesnt contain type {wrapped_type:#?}")
            }
        });

        Self {
            name: name.to_string(),
            r#type: wrapped_type.r#type(),
            type_index,
            ..Default::default()
        }
    }

    pub fn size(&self, schema: &Schema) -> usize {
        schema
            .type_size(self.r#type, self.type_index)
            .expect("Provided schema does not contain this type!")
    }

    pub fn align(&self, schema: &Schema) -> usize {
        schema
            .type_align(self.r#type, self.type_index)
            .expect("Provided schema does not contain this type!")
    }

    pub fn layout(&self, schema: &Schema) -> Layout {
        Layout::from_size_align(self.size(schema), self.align(schema)).unwrap()
    }

    pub fn is_copy(&self, schema: &Schema) -> bool {
        schema
            .is_copy(self.r#type, self.type_index)
            .expect("Provided schema does not contain this type!")
    }

    pub fn type_name<'a>(&self, schema: &'a Schema) -> Cow<'a, str> {
        schema.type_name(self.r#type, self.type_index)
    }
}
