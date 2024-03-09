use crate::types::primitives::PRIMITIVES;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnumType {
    pub name: String,
    pub variants: Vec<String>,
    pub repr_type_index: usize,
}
impl Default for EnumType {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            variants: vec![],
            repr_type_index: PRIMITIVES.index_of("u8").unwrap(),
        }
    }
}
impl EnumType {
    pub fn new(name: &str, variants: &[&str], repr_type_index: usize) -> Self {
        Self {
            name: name.to_string(),
            variants: variants.iter().map(ToString::to_string).collect(),
            repr_type_index,
        }
    }

    pub fn default_with_name(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }

    pub fn size(&self) -> usize {
        PRIMITIVES.index(self.repr_type_index).unwrap().size
    }
}
