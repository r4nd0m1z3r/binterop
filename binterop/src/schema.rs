use crate::primitives::PRIMITIVES;
use crate::types::{DataType, EnumType, UnionType};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum Type {
    #[default]
    Primitive,
    Data,
    Enum,
    Union,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Schema {
    pub root_type_index: usize,
    pub types: Vec<DataType>,
    pub enums: Vec<EnumType>,
    pub unions: Vec<UnionType>,
}
impl Schema {
    pub fn new(
        root_type_index: usize,
        types: &[DataType],
        enums: &[EnumType],
        unions: &[UnionType],
    ) -> Self {
        Self {
            root_type_index,
            types: types.to_vec(),
            enums: enums.to_vec(),
            unions: unions.to_vec(),
        }
    }

    pub fn type_name(&self, index: usize, r#type: Type) -> String {
        match r#type {
            Type::Primitive => PRIMITIVES.name_of(index).unwrap().to_string(),
            Type::Data => self.types[index].name.clone(),
            Type::Enum => self.enums[index].name.clone(),
            Type::Union => self.unions[index].name.clone(),
        }
    }

    pub fn type_size(&self, index: usize, r#type: Type) -> usize {
        match r#type {
            Type::Primitive => PRIMITIVES.index(index).unwrap().size,
            Type::Data => self.types[index].size(self),
            Type::Enum => self.enums[index].size(),
            Type::Union => self.unions[index].size(self),
        }
    }

    pub fn type_data_by_name(&self, name: &str) -> Result<(usize, Type, usize), String> {
        if let Some(index) = PRIMITIVES.index_of(name) {
            let type_size = PRIMITIVES[name].size;
            return Ok((index, Type::Primitive, type_size));
        }

        if let Some(index) = self
            .types
            .iter()
            .enumerate()
            .find(|(_, data_type)| data_type.name == name)
            .map(|(index, _)| index)
        {
            let type_size = self.type_size(index, Type::Data);
            return Ok((index, Type::Data, type_size));
        }

        if let Some(index) = self
            .enums
            .iter()
            .enumerate()
            .find(|(_, enum_type)| enum_type.name == *name)
            .map(|(index, _)| index)
        {
            let type_size = self.type_size(index, Type::Enum);
            return Ok((index, Type::Enum, type_size));
        }

        if let Some(index) = self
            .unions
            .iter()
            .enumerate()
            .find(|(_, union_type)| union_type.name == *name)
            .map(|(index, _)| index)
        {
            let type_size = self.type_size(index, Type::Union);
            return Ok((index, Type::Union, type_size));
        }

        let available_type_names = self
            .types
            .iter()
            .map(|data_type| data_type.name.clone())
            .collect::<Vec<_>>();
        let available_enum_names = self
            .enums
            .iter()
            .map(|enum_type| enum_type.name.clone())
            .collect::<Vec<_>>();
        let available_union_names = self
            .unions
            .iter()
            .map(|union_type| union_type.name.clone())
            .collect::<Vec<_>>();

        Err(format!("Failed to find type with name {name:?}!\n\tAvailable types: {available_type_names:?}\n\tAvailable enums: {available_enum_names:?}\n\tAvailable unions: {available_union_names:?}"))
    }

    pub fn root_size(&self) -> usize {
        self.types[self.root_type_index].size(self)
    }

    pub fn allocate_root(&self, count: usize) -> Vec<u8> {
        vec![0; self.root_size() * count]
    }
}
