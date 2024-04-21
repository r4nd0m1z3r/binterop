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
    pub is_copy: bool,
}
impl TypeData {
    pub fn new(index: usize, r#type: Type, size: usize, is_copy: bool) -> Self {
        Self {
            index,
            r#type,
            size,
            is_copy,
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
    Vector,
    Pointer,
}
