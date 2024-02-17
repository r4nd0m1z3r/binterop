use crate::types::{DataType, EnumType};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize)]
pub enum Type {
    #[default]
    Primitive,
    Data,
    Enum,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Schema {
    pub root_type_index: usize,
    pub types: Vec<DataType>,
    pub enums: Vec<EnumType>,
}
impl Schema {
    pub fn new(root_type_index: usize, types: &[DataType], enums: &[EnumType]) -> Self {
        Self {
            root_type_index,
            types: types.to_vec(),
            enums: enums.to_vec(),
        }
    }

    pub fn root_size(&self) -> usize {
        self.types[self.root_type_index].size(self)
    }

    pub fn allocate_root(&self, count: usize) -> Vec<u8> {
        vec![0; self.root_size() * count]
    }
}
