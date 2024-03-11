use crate::schema::{Schema, Type};
use crate::types::primitives::PRIMITIVES;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnionType {
    pub name: String,
    pub possible_types: Vec<(usize, Type)>,
    pub repr_type_index: usize,
}
impl Default for UnionType {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            possible_types: vec![],
            repr_type_index: PRIMITIVES.index_of("u8").unwrap(),
        }
    }
}
impl UnionType {
    pub fn new(
        name: &str,
        possible_types: &[(usize, Type)],
        variant_repr_type_index: usize,
    ) -> Self {
        Self {
            name: name.to_string(),
            possible_types: possible_types.to_vec(),
            repr_type_index: variant_repr_type_index,
        }
    }

    pub fn default_with_name(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }

    pub fn size(&self, schema: &Schema) -> usize {
        let repr_type_size = PRIMITIVES.index(self.repr_type_index).unwrap().size;
        let max_possible_type_size = self
            .possible_types
            .iter()
            .map(|(index, r#type)| schema.type_size(*r#type, *index))
            .max()
            .unwrap();

        repr_type_size + max_possible_type_size
    }
}
