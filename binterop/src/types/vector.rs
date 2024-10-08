use crate::types::pointer::PointerType;
use crate::types::Type;
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
}
