use crate::schema::{Schema, Type};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Default, Debug, Serialize, Deserialize)]
pub struct ArrayType {
    pub inner_type: Type,
    pub inner_type_index: usize,
    pub len: usize,
}
impl ArrayType {
    pub fn new(inner_type: Type, inner_type_index: usize, len: usize) -> Self {
        Self {
            inner_type,
            inner_type_index,
            len,
        }
    }

    pub fn size(&self, schema: &Schema) -> usize {
        let inner_type_size = schema
            .type_size(self.inner_type, self.inner_type_index)
            .expect("Provided schema does not contain inner type!");

        inner_type_size * self.len
    }
}
