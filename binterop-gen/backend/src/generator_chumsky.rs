use binterop::{
    field::Field,
    schema::Schema,
    types::{
        array::ArrayType, data::DataType, pointer::PointerType, primitives::PRIMITIVES,
        r#enum::EnumType, union::UnionType, vector::VectorType, Type, TypeData,
    },
};

use crate::tokenizer_chumsky as tokenizer;
use tokenizer::Token;

fn lookup_type_data(schema: &mut Schema, r#type: &tokenizer::Type) -> Option<TypeData> {
    match r#type {
        tokenizer::Type::Named(name) => {
            if let Some(index) = PRIMITIVES.index_of(name) {
                return schema.type_data(index, Type::Primitive).ok();
            }
            if let Some(index) = schema.types.iter().position(|r#type| &r#type.name == name) {
                return schema.type_data(index, Type::Data).ok();
            }
            if let Some(index) = schema.enums.iter().position(|r#type| &r#type.name == name) {
                return schema.type_data(index, Type::Enum).ok();
            }
            if let Some(index) = schema.unions.iter().position(|r#type| &r#type.name == name) {
                return schema.type_data(index, Type::Union).ok();
            }
            if let Some(index) = schema
                .functions
                .iter()
                .position(|r#type| &r#type.name == name)
            {
                return schema.type_data(index, Type::Function).ok();
            }
            if name == &"String" {
                return schema.type_data(0, Type::String).ok();
            }
        }
        tokenizer::Type::Array(inner_type, size) => {
            let inner_type_data = lookup_type_data(schema, &inner_type)?;
            let index = schema
                .arrays
                .iter()
                .position(|array| {
                    array.inner_type == inner_type_data.r#type
                        && array.inner_type_index == inner_type_data.index
                        && array.len == *size
                })
                .or_else(|| {
                    let index = schema.arrays.len();
                    schema.arrays.push(ArrayType::new(
                        inner_type_data.r#type,
                        inner_type_data.index,
                        *size,
                    ));
                    Some(index)
                })?;

            return schema.type_data(index, Type::Array).ok();
        }
        tokenizer::Type::Vector(inner_type) => {
            let inner_type_data = lookup_type_data(schema, &inner_type)?;
            let index = schema
                .vectors
                .iter()
                .position(|vector| {
                    vector.inner_type == inner_type_data.r#type
                        && vector.inner_type_index == inner_type_data.index
                })
                .or_else(|| {
                    let index = schema.vectors.len();
                    schema.vectors.push(VectorType::new(
                        inner_type_data.r#type,
                        inner_type_data.index,
                    ));
                    Some(index)
                })?;

            return schema.type_data(index, Type::Vector).ok();
        }
        tokenizer::Type::Pointer(pointee_type) => {
            let pointee_type_data = lookup_type_data(schema, &pointee_type)?;
            let index = schema
                .pointers
                .iter()
                .position(|pointer| {
                    pointer.inner_type == pointee_type_data.r#type
                        && pointer.inner_type_index == pointee_type_data.index
                })
                .or_else(|| {
                    let index = schema.pointers.len();
                    schema.pointers.push(PointerType::new(
                        pointee_type_data.r#type,
                        pointee_type_data.index,
                    ));
                    Some(index)
                })?;

            return schema.type_data(index, Type::Pointer).ok();
        }
    }

    None
}

pub fn generate_schema<'a>(tokens: impl Iterator<Item = &'a Token<'a>>) -> Result<Schema, String> {
    let mut schema = Schema::default();

    for token in tokens {
        match token {
            Token::Struct(name, fields) => {
                let mut data_type = DataType::default_with_name(name);
                let mut current_offset = 0;

                for (name, r#type) in fields {
                    let type_data = lookup_type_data(&mut schema, &r#type)
                        .ok_or(format!("Failed to lookup type {type:?} for field {name}"))?;

                    let field =
                        Field::new(name, type_data.r#type, type_data.index, current_offset, 0);
                    current_offset += field.size(&schema);

                    data_type.fields.push(field);
                }

                schema.types.push(data_type);
            }
            Token::Enum(name, variants) => {
                let mut enum_type = EnumType::default_with_name(name);
                enum_type.variants = variants.iter().map(|variant| variant.to_string()).collect();

                schema.enums.push(enum_type);
            }
            Token::Union(name, variants) => {
                let mut union_type = UnionType::default_with_name(name);
                let possible_types = variants
                    .into_iter()
                    .map(|variant| {
                        let variant_type_data =
                            lookup_type_data(&mut schema, &tokenizer::Type::Named(variant)).ok_or(
                                format!("Failed to lookup type named {variant} in union {name}"),
                            )?;

                        Ok((variant_type_data.index, variant_type_data.r#type))
                    })
                    .collect::<Result<Vec<_>, String>>()?;

                union_type.possible_types = possible_types;

                schema.unions.push(union_type);
            }
            Token::Include(path, tokens) => {
                let mut include_schema = generate_schema(tokens.iter())
                    .map_err(|err| format!("Failed to generate schema for {path:?}! Err: {err}"))?;
                schema.append(&mut include_schema);
            }
        }
    }

    Ok(schema)
}
