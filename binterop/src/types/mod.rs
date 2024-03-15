use serde::{Deserialize, Serialize};

pub mod array;
pub mod data;
pub mod r#enum;
pub mod pointer;
pub mod primitives;
pub mod union;
pub mod vector;

pub struct TypeData {
    pub index: usize,
    pub r#type: Type,
    pub size: usize,
}
impl TypeData {
    pub fn new(index: usize, r#type: Type, size: usize) -> Self {
        Self {
            index,
            r#type,
            size,
        }
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum Type {
    #[default]
    Primitive,
    Data,
    Enum,
    Union,
    Array,
    Pointer,
}
