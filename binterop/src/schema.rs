use crate::types::array::ArrayType;
use crate::types::data::DataType;
use crate::types::pointer::PointerType;
use crate::types::primitives::PRIMITIVES;
use crate::types::r#enum::EnumType;
use crate::types::union::UnionType;
use crate::types::vector::VectorType;
use crate::types::{Type, TypeData};
use crate::WrappedType;
use serde::{Deserialize, Serialize};
use std::alloc::Layout;
use std::borrow::Cow;
use std::mem::{align_of, size_of};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Schema {
    pub is_packed: bool,
    pub types: Vec<DataType>,
    pub enums: Vec<EnumType>,
    pub unions: Vec<UnionType>,
    pub arrays: Vec<ArrayType>,
    pub pointers: Vec<PointerType>,
    pub vectors: Vec<VectorType>,
}
impl Schema {
    pub fn new(
        is_packed: bool,
        types: &[DataType],
        enums: &[EnumType],
        unions: &[UnionType],
        arrays: &[ArrayType],
        pointers: &[PointerType],
        vectors: &[VectorType],
    ) -> Self {
        Self {
            is_packed,
            types: types.to_vec(),
            enums: enums.to_vec(),
            unions: unions.to_vec(),
            arrays: arrays.to_vec(),
            pointers: pointers.to_vec(),
            vectors: vectors.to_vec(),
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
            Type::Vector => {
                let VectorType {
                    inner_type,
                    inner_type_index,
                } = self.vectors[index];
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
            Type::String => Cow::Owned("String".to_string()),
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
            Type::Vector | Type::String => Some(VectorType::size()),
            Type::Pointer => Some(PointerType::size()),
        }
    }

    pub fn is_copy(&self, r#type: Type, index: usize) -> Option<bool> {
        match r#type {
            Type::Primitive => Some(true),
            Type::Data => self
                .types
                .get(index)
                .map(|data_type| data_type.is_copy(self)),
            Type::Enum => Some(true),
            Type::Union => self
                .unions
                .get(index)
                .map(|union_type| union_type.is_copy(self)),
            Type::Array => Some(true),
            Type::Vector | Type::String => Some(false),
            Type::Pointer => Some(true),
        }
    }

    pub fn type_align(&self, r#type: Type, index: usize) -> Option<usize> {
        match r#type {
            Type::Primitive => PRIMITIVES.index(index).map(|primitive| primitive.align),
            Type::Data => self.types.get(index).map(|data_type| data_type.align(self)),
            Type::Enum => Some(EnumType::align()),
            Type::Union => self
                .unions
                .get(index)
                .map(|union_type| union_type.align(self)),
            Type::Array => self
                .arrays
                .get(index)
                .map(|array_type| array_type.align(self)),
            Type::Vector | Type::String => Some(
                Layout::from_size_align(size_of::<u64>() * 3, size_of::<u64>())
                    .unwrap()
                    .align(),
            ),
            Type::Pointer => Some(align_of::<u64>()),
        }
    }

    pub fn wrapped_type_index(&self, wrapped_type: &WrappedType) -> Option<usize> {
        match wrapped_type {
            WrappedType::Array(array_type) => self
                .arrays
                .iter()
                .position(|schema_array_type| schema_array_type == array_type),
            WrappedType::Data(data_type) => self
                .types
                .iter()
                .position(|schema_data_type| schema_data_type.name == data_type.name),
            WrappedType::Enum(enum_type) => self
                .enums
                .iter()
                .position(|schema_enum_type| schema_enum_type.name == enum_type.name),
            WrappedType::Pointer(pointer_type) => self
                .pointers
                .iter()
                .position(|schema_pointer_type| schema_pointer_type == pointer_type),
            WrappedType::Primitive(primitive_type) => PRIMITIVES.index_of(primitive_type.name),
            WrappedType::Union(union_type) => self
                .unions
                .iter()
                .position(|schema_union_type| schema_union_type.name == union_type.name),
            WrappedType::Vector(vector_type) => self
                .vectors
                .iter()
                .position(|schema_vector_type| schema_vector_type == vector_type),
            WrappedType::String => None,
        }
    }

    pub fn type_data_by_name(&self, name: &str) -> Result<TypeData, String> {
        if name == "String" {
            return Ok(TypeData::new(0, Type::String, VectorType::size(), false));
        }

        if let Some(index) = PRIMITIVES.index_of(name) {
            let type_size = PRIMITIVES[name].size;
            return Ok(TypeData::new(index, Type::Primitive, type_size, true));
        }

        if let Some((index, _)) = self
            .types
            .iter()
            .enumerate()
            .find(|(_, data_type)| data_type.name == name)
        {
            let type_size = self.type_size(Type::Data, index).unwrap();
            let is_copy = self.is_copy(Type::Data, index).unwrap();
            return Ok(TypeData::new(index, Type::Data, type_size, is_copy));
        }

        if let Some(index) = self
            .enums
            .iter()
            .enumerate()
            .find(|(_, enum_type)| enum_type.name == *name)
            .map(|(index, _)| index)
        {
            let type_size = self.type_size(Type::Enum, index).unwrap();
            return Ok(TypeData::new(index, Type::Enum, type_size, true));
        }

        if let Some(index) = self
            .unions
            .iter()
            .enumerate()
            .find(|(_, union_type)| union_type.name == *name)
            .map(|(index, _)| index)
        {
            let type_size = self.type_size(Type::Union, index).unwrap();
            let is_copy = self.is_copy(Type::Union, index).unwrap();
            return Ok(TypeData::new(index, Type::Union, type_size, is_copy));
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

    pub fn append(&mut self, schema: &mut Self) {
        self.is_packed |= schema.is_packed;

        self.types.append(&mut schema.types);
        self.enums.append(&mut schema.enums);
        self.unions.append(&mut schema.unions);
        self.arrays.append(&mut schema.arrays);
        self.pointers.append(&mut schema.pointers);
        self.vectors.append(&mut schema.vectors);
    }
}
