use crate::schema::Schema;
use crate::types::Type;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Copy, Clone, Default, Debug, Serialize, Deserialize)]
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

    pub fn align(&self, schema: &Schema) -> usize {
        schema
            .type_align(self.inner_type, self.inner_type_index)
            .expect("Provided schema does not contain inner type!")
    }

    pub fn parse(s: &str, schema: &mut Schema) -> Result<Self, String> {
        let mut s_split = s[1..(s.len() - 1)].split(':');

        let inner_type_data = if let Some(type_name) = s_split.next() {
            schema.type_data_by_name(type_name)?
        } else {
            return Err(format!("Failed to split type name from {s}"));
        };

        let len = if let Some(len_str) = s_split.next() {
            len_str.parse::<usize>().map_err(|err| err.to_string())
        } else {
            Err(format!("Failed to split length from {s}"))
        }?;

        Ok(Self::new(
            inner_type_data.r#type,
            inner_type_data.index,
            len,
        ))
    }
}
