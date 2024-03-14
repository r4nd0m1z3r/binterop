use crate::schema::Type;
use serde::{Deserialize, Serialize};
use std::mem::size_of;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct PointerType {
    pub inner_type: Type,
    pub inner_type_index: usize,
}
impl PointerType {
    pub fn new(inner_type: Type, inner_type_index: usize) -> Self {
        Self {
            inner_type,
            inner_type_index,
        }
    }

    pub fn size() -> usize {
        size_of::<u64>()
    }
}
