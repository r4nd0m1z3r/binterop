use array::ArrayType;
use data::DataType;
use pointer::PointerType;
use primitives::PrimitiveType;
use r#enum::EnumType;
use serde::{Deserialize, Serialize};
use union::UnionType;
use vector::VectorType;

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
    String,
}

#[derive(Clone, Debug)]
pub enum WrappedType {
    Array(ArrayType),
    Data(DataType),
    Enum(EnumType),
    Pointer(PointerType),
    Primitive(PrimitiveType),
    Union(UnionType),
    Vector(VectorType),
    String,
}
impl WrappedType {
    pub fn r#type(&self) -> Type {
        match self {
            WrappedType::Array(_) => Type::Array,
            WrappedType::Data(_) => Type::Data,
            WrappedType::Enum(_) => Type::Enum,
            WrappedType::Pointer(_) => Type::Pointer,
            WrappedType::Primitive(_) => Type::Primitive,
            WrappedType::Union(_) => Type::Union,
            WrappedType::Vector(_) => Type::Vector,
            WrappedType::String => Type::String,
        }
    }
}
