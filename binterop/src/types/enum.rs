use serde::{Deserialize, Serialize};
use std::mem::{align_of, size_of};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnumType {
    pub name: String,
    pub variants: Vec<String>,
    pub attributes: Vec<(String, String)>,
}
impl Default for EnumType {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            variants: Vec::new(),
            attributes: Vec::new(),
        }
    }
}
impl EnumType {
    pub fn new(name: &str, variants: &[&str], attributes: &[(String, String)]) -> Self {
        Self {
            name: name.to_string(),
            variants: variants.iter().map(ToString::to_string).collect(),
            attributes: attributes.to_vec(),
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
