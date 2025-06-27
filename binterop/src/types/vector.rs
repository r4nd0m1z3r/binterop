use crate::types::Type;
use crate::{schema::Schema, types::pointer::PointerType};
use serde::{Deserialize, Serialize};
use std::mem::size_of;

#[derive(PartialEq, Eq, Copy, Clone, Default, Debug, Serialize, Deserialize)]
pub struct VectorType {
    pub inner_type: Type,
    pub inner_type_index: usize,
}
impl VectorType {
    pub fn new(inner_type: Type, inner_type_index: usize) -> Self {
        Self {
            inner_type,
            inner_type_index,
        }
    }

    pub fn size() -> usize {
        PointerType::size() + size_of::<u64>() * 2
    }

    pub fn is_copy() -> bool {
        false
    }

    pub fn parse(s: &str, schema: &mut Schema) -> Result<Self, String> {
        let inner_type_name = &s[1..(s.len() - 1)];
        let inner_type_data = schema.type_data_by_name(inner_type_name)?;

        Ok(Self::new(inner_type_data.r#type, inner_type_data.index))
    }
}
