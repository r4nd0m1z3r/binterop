use serde::{Deserialize, Serialize};

use crate::types::{pointer::PointerType, TypeData};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Arg {
    pub name: String,
    pub r#type: Option<TypeData>,
}
impl Arg {
    pub fn new(name: String, r#type: Option<TypeData>) -> Self {
        Arg { name, r#type }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct FunctionType {
    pub name: String,
    pub args: Vec<Arg>,
    pub return_type: Option<TypeData>,
}
impl FunctionType {
    pub fn new(name: String, args: Vec<Arg>, return_type: Option<TypeData>) -> Self {
        FunctionType {
            name,
            args,
            return_type,
        }
    }

    pub fn default_with_name(name: &str) -> Self {
        FunctionType {
            name: name.to_string(),
            ..Default::default()
        }
    }

    pub fn size() -> usize {
        PointerType::size()
    }
}
