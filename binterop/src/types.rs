use crate::primitives::PRIMITIVES;
use crate::schema::{Schema, Type};
use crate::Field;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct PrimitiveType {
    pub name: &'static str,
    pub size: usize,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DataType {
    pub name: String,
    pub fields: Vec<Field>,
}
impl DataType {
    pub fn new(schema: &Schema, name: &str, field_data: &[(&str, Type, usize)]) -> Self {
        let fields = field_data
            .iter()
            .copied()
            .enumerate()
            .map(|(index, (name, r#type, type_index))| {
                let type_size = match r#type {
                    Type::Primitive => PRIMITIVES.index(type_index).map(|(_, v)| v.size).unwrap(),
                    Type::Data => schema.types[type_index].size(schema),
                    Type::Enum => schema.enums[type_index].size(),
                };

                Field::new(name, r#type, type_index, index * type_size)
            })
            .collect();

        Self {
            name: name.to_string(),
            fields,
        }
    }

    pub fn default_with_name(name: &str) -> Self {
        Self {
            name: name.to_string(),
            fields: vec![],
        }
    }

    pub fn from_primitives(name: &str, field_data: &[(&str, PrimitiveType)]) -> Self {
        let mut previous_offset = 0;
        let fields = field_data
            .iter()
            .map(|(name, r#type)| {
                let primitive_index = PRIMITIVES.get_index(r#type.name).unwrap();
                let primitive_size = PRIMITIVES
                    .index(primitive_index)
                    .map(|(_, v)| v.size)
                    .unwrap();

                let field = Field::new(name, Type::Primitive, primitive_index, previous_offset);

                previous_offset += primitive_size;

                field
            })
            .collect();

        Self {
            name: name.to_string(),
            fields,
        }
    }

    pub fn from_fields(name: &str, fields: &[Field]) -> Self {
        Self {
            name: name.to_string(),
            fields: fields.to_vec(),
        }
    }

    pub fn size(&self, schema: &Schema) -> usize {
        self.fields.iter().map(|field| field.size(schema)).sum()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnumType {
    pub name: String,
    pub variants: Vec<String>,
    pub repr_type_index: usize,
}
impl Default for EnumType {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            variants: vec![],
            repr_type_index: PRIMITIVES.get_index("u8").unwrap(),
        }
    }
}
impl EnumType {
    pub fn new(name: &str, variants: &[&str], repr_type_index: usize) -> Self {
        Self {
            name: name.to_string(),
            variants: variants.iter().map(ToString::to_string).collect(),
            repr_type_index,
        }
    }

    pub fn default_with_name(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }

    pub fn size(&self) -> usize {
        PRIMITIVES
            .index(self.repr_type_index)
            .map(|(_, v)| v.size)
            .unwrap()
    }
}
