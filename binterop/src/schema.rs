use crate::types::array::ArrayType;
use crate::types::data::DataType;
use crate::types::heap_array::HeapArrayType;
use crate::types::pointer::PointerType;
use crate::types::primitives::PRIMITIVES;
use crate::types::r#enum::EnumType;
use crate::types::union::UnionType;
use crate::types::{Type, TypeData};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Schema {
    pub root_type_index: usize,
    pub types: Vec<DataType>,
    pub enums: Vec<EnumType>,
    pub unions: Vec<UnionType>,
    pub arrays: Vec<ArrayType>,
    pub pointers: Vec<PointerType>,
    pub heap_arrays: Vec<HeapArrayType>,
}
impl Schema {
    pub fn new(
        root_type_index: usize,
        types: &[DataType],
        enums: &[EnumType],
        unions: &[UnionType],
        arrays: &[ArrayType],
        pointers: &[PointerType],
        heap_arrays: &[HeapArrayType],
    ) -> Self {
        Self {
            root_type_index,
            types: types.to_vec(),
            enums: enums.to_vec(),
            unions: unions.to_vec(),
            arrays: arrays.to_vec(),
            pointers: pointers.to_vec(),
            heap_arrays: heap_arrays.to_vec(),
        }
    }

    pub fn type_name(&self, r#type: Type, index: usize) -> Cow<str> {
        match r#type {
            Type::Primitive => Cow::Borrowed(PRIMITIVES.name_of(index).unwrap()),
            Type::Data => Cow::Borrowed(&self.types[index].name),
            Type::Enum => Cow::Borrowed(&self.enums[index].name),
            Type::Union => Cow::Borrowed(&self.unions[index].name),
            Type::Array => {
                let ArrayType {
                    inner_type,
                    inner_type_index,
                    len,
                } = self.arrays[index];
                let inner_type_name = self.type_name(inner_type, inner_type_index);

                Cow::Owned(format!("[{inner_type_name}:{len}]"))
            }
            Type::HeapArray => {
                let HeapArrayType {
                    inner_type,
                    inner_type_index,
                } = self.heap_arrays[index];
                let inner_type_name = self.type_name(inner_type, inner_type_index);

                Cow::Owned(format!("<{inner_type_name}>"))
            }
            Type::Pointer => {
                let PointerType {
                    inner_type,
                    inner_type_index,
                } = self.pointers[index];
                let inner_type_name = self.type_name(inner_type, inner_type_index);

                Cow::Owned(format!("{inner_type_name}*"))
            }
        }
    }

    pub fn type_size(&self, r#type: Type, index: usize) -> Option<usize> {
        match r#type {
            Type::Primitive => PRIMITIVES.index(index).map(|primitive| primitive.size),
            Type::Data => self.types.get(index).map(|data_type| data_type.size(self)),
            Type::Enum => Some(EnumType::size()),
            Type::Union => self
                .unions
                .get(index)
                .map(|union_type| union_type.size(self)),
            Type::Array => self
                .arrays
                .get(index)
                .map(|array_type| array_type.size(self)),
            Type::HeapArray => Some(HeapArrayType::size()),
            Type::Pointer => Some(PointerType::size()),
        }
    }

    pub fn type_data_by_name(&self, name: &str) -> Result<TypeData, String> {
        if let Some(index) = PRIMITIVES.index_of(name) {
            let type_size = PRIMITIVES[name].size;
            return Ok(TypeData::new(index, Type::Primitive, type_size));
        }

        if let Some(index) = self
            .types
            .iter()
            .enumerate()
            .find(|(_, data_type)| data_type.name == name)
            .map(|(index, _)| index)
        {
            let type_size = self.type_size(Type::Data, index).unwrap();
            return Ok(TypeData::new(index, Type::Data, type_size));
        }

        if let Some(index) = self
            .enums
            .iter()
            .enumerate()
            .find(|(_, enum_type)| enum_type.name == *name)
            .map(|(index, _)| index)
        {
            let type_size = self.type_size(Type::Enum, index).unwrap();
            return Ok(TypeData::new(index, Type::Enum, type_size));
        }

        if let Some(index) = self
            .unions
            .iter()
            .enumerate()
            .find(|(_, union_type)| union_type.name == *name)
            .map(|(index, _)| index)
        {
            let type_size = self.type_size(Type::Union, index).unwrap();
            return Ok(TypeData::new(index, Type::Union, type_size));
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
