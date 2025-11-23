use crate::field::Field;
use crate::schema::Schema;
use crate::types::primitives::{PrimitiveType, PRIMITIVES};
use crate::types::Type;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DataType {
    pub name: String,
    pub fields: Vec<Field>,
    pub attributes: Vec<(String, String)>,
}
impl DataType {
    pub fn new(schema: &Schema, name: &str, field_data: &[(&str, Type, usize)], attributes: &[(String, String)]) -> Self {
        let fields = field_data
            .iter()
            .copied()
            .enumerate()
            .map(|(index, (name, r#type, type_index))| {
                let type_size = schema.type_size(r#type, index).unwrap_or_else(|| {
                    panic!("Provided schema does not contain type {type:?} with index {index}!")
                });

                Field::new(name, r#type, type_index, index * type_size, 0, &[])
            })
            .collect();

        Self {
            name: name.to_string(),
            fields,
            attributes: attributes.to_vec(),
        }
    }

    pub fn default_with_name(name: &str) -> Self {
        Self {
            name: name.to_string(),
            fields: Vec::new(),
            attributes: Vec::new(),
        }
    }

    pub fn from_primitives(name: &str, field_data: &[(&str, PrimitiveType)]) -> Self {
        let mut previous_offset = 0;
        let fields = field_data
            .iter()
            .map(|(name, r#type)| {
                let primitive_index = PRIMITIVES.index_of(r#type.name).unwrap();
                let primitive_size = PRIMITIVES[name].size;

                let field = Field::new(name, Type::Primitive, primitive_index, previous_offset, 0, &[]);

                previous_offset += primitive_size;

                field
            })
            .collect();

        Self {
            name: name.to_string(),
            fields,
            attributes: Vec::new(),
        }
    }

    pub fn from_fields(name: &str, fields: &[Field]) -> Self {
        Self {
            name: name.to_string(),
            fields: fields.to_vec(),
            attributes: Vec::new(),
        }
    }

    pub fn size(&self, schema: &Schema) -> usize {
        let self_indices = self
            .fields
            .iter()
            .enumerate()
            .filter_map(|(index, field)| {
                if field.type_name(schema) == self.name {
                    Some(index)
                } else {
                    None
                }
            })
            .collect::<SmallVec<[usize; 32]>>();

        let self_size: usize = self
            .fields
            .iter()
            .enumerate()
            .filter_map(|(index, field)| {
                if !self_indices.contains(&index) {
                    Some(field.size(schema))
                } else {
                    None
                }
            })
            .sum();

        self_size + (self_indices.len() * self_size)
    }

    pub fn align(&self, schema: &Schema) -> usize {
        let self_indices = self
            .fields
            .iter()
            .enumerate()
            .filter_map(|(index, field)| {
                if field.type_name(schema) == self.name {
                    Some(index)
                } else {
                    None
                }
            })
            .collect::<SmallVec<[usize; 32]>>();

        self.fields
            .iter()
            .enumerate()
            .filter_map(|(index, field)| {
                if !self_indices.contains(&index) {
                    Some(field.align(schema))
                } else {
                    None
                }
            })
            .max()
            .unwrap_or(1)
    }

    pub fn is_copy(&self, schema: &Schema) -> bool {
        self.fields.iter().all(|field| field.is_copy(schema))
    }
}
