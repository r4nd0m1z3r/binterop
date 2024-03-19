use crate::schema::Schema;
use crate::types::Type;
use serde::{Deserialize, Serialize};

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

    pub fn size(&self, schema: &Schema) -> usize {
        schema
            .type_size(self.r#type, self.type_index)
            .expect("Provided schema does not contain this type!")
    }
}
