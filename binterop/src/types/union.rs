use crate::schema::Schema;
use crate::types::Type;
use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::mem::{align_of, size_of};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnionType {
    pub name: String,
    pub possible_types: Vec<(usize, Type)>,
    pub attributes: Vec<(String, String)>,
}
impl Default for UnionType {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            possible_types: vec![],
            attributes: Vec::new(),
        }
    }
}
impl UnionType {
    pub fn new(name: &str, possible_types: &[(usize, Type)], attributes: &[(String, String)]) -> Self {
        Self {
            name: name.to_string(),
            possible_types: possible_types.to_vec(),
            attributes: attributes.to_vec(),
        }
    }

    pub fn default_with_name(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }

    pub fn size(&self, schema: &Schema) -> usize {
        let max_possible_type_size = self
            .possible_types
            .iter()
            .map(|&(index, r#type)| {
                schema.type_size(r#type, index).unwrap_or_else(|| {
                    panic!("Provided schema does not contain type {type:?} with index {index}!")
                })
            })
            .max()
            .unwrap();

        size_of::<i32>() + max_possible_type_size
    }

    pub fn align(&self, schema: &Schema) -> usize {
        let repr_type_align = align_of::<i32>();
        let max_possible_type_align = self
            .possible_types
            .iter()
            .map(|&(index, r#type)| {
                schema.type_align(r#type, index).unwrap_or_else(|| {
                    panic!("Provided schema does not contain type {type:?} with index {index}!")
                })
            })
            .max()
            .unwrap();

        max(repr_type_align, max_possible_type_align)
    }

    pub fn is_copy(&self, schema: &Schema) -> bool {
        self.possible_types
            .iter()
            .all(|&(index, r#type)| schema.is_copy(r#type, index).unwrap_or_default())
    }
}
