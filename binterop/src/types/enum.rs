use serde::{Deserialize, Serialize};
use std::mem::{align_of, size_of};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnumType {
    pub name: String,
    pub variants: Vec<String>,
}
impl Default for EnumType {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            variants: vec![],
        }
    }
}
impl EnumType {
    pub fn new(name: &str, variants: &[&str]) -> Self {
        Self {
            name: name.to_string(),
            variants: variants.iter().map(ToString::to_string).collect(),
        }
    }

    pub fn default_with_name(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }

    pub fn size() -> usize {
        size_of::<i32>()
    }

    pub fn align() -> usize {
        align_of::<i32>()
    }
}
