use std::collections::VecDeque;

use binterop::{
    field::Field,
    schema::Schema,
    types::{
        Type, TypeData,
        array::ArrayType,
        data::DataType,
        r#enum::EnumType,
        function::{Arg, FunctionType},
        pointer::PointerType,
        primitives::PRIMITIVES,
        union::UnionType,
        vector::VectorType,
    },
};

use crate::tokenizer::{self, Token};

fn lookup_type_data(
    defined_type_name: &str,
    defined_type: Type,
    schema: &mut Schema,
    r#type: &tokenizer::Type,
) -> Option<TypeData> {
    match r#type {
        tokenizer::Type::Named(name) => {
            if *name == defined_type_name {
                let recursive_type_data = match defined_type {
                    Type::Data => TypeData::new(schema.types.len(), Type::Data, 0, false),
                    Type::Union => TypeData::new(schema.unions.len(), Type::Union, 0, false),
                    Type::Function => {
                        TypeData::new(schema.functions.len(), Type::Function, 0, false)
                    }
                    _ => unreachable!("Cannot define {defined_type:?}!"),
                };

                return Some(recursive_type_data);
            }

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
            let inner_type_data =
                lookup_type_data(defined_type_name, defined_type, schema, &inner_type)?;
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
            let inner_type_data =
                lookup_type_data(defined_type_name, defined_type, schema, &inner_type)?;
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
            let pointee_type_data =
                lookup_type_data(defined_type_name, defined_type, schema, &pointee_type)?;
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

pub fn generate_schema<'a>(tokens: &VecDeque<Token<'a>>) -> Result<Schema, String> {
    let mut schema = Schema::default();

    let mut recursive_fields_indices = Vec::new();

    for token in tokens {
        match token {
            Token::Struct(struct_name, fields) => {
                let mut data_type = DataType::default_with_name(struct_name);
                let mut current_offset = 0;

                for (field_name, r#type) in fields {
                    if let tokenizer::Type::Named(type_name) = r#type
                        && type_name == struct_name
                    {
                        recursive_fields_indices.push(data_type.fields.len());
                    }

                    let type_data = lookup_type_data(struct_name, Type::Data, &mut schema, &r#type)
                        .ok_or(format!(
                            "Failed to lookup type {type:?} for field {field_name}"
                        ))?;

                    let field = Field::new(
                        field_name,
                        type_data.r#type,
                        type_data.index,
                        current_offset,
                        0,
                    );
                    current_offset += type_data.size;

                    data_type.fields.push(field);
                }
                schema.types.push(data_type);

                let data_type = schema.types.last().unwrap();
                let self_size = data_type.size(&schema);
                let data_type = schema.types.last_mut().unwrap();

                for &field_index in &recursive_fields_indices {
                    let next_fields = data_type.fields[field_index + 1..].iter_mut();
                    for field in next_fields {
                        field.offset += self_size;
                    }
                }
                recursive_fields_indices.clear();
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
                        let variant_type_data = lookup_type_data(
                            name,
                            Type::Union,
                            &mut schema,
                            &tokenizer::Type::Named(variant),
                        )
                        .ok_or(format!(
                            "Failed to lookup type named {variant} in union {name}"
                        ))?;

                        Ok((variant_type_data.index, variant_type_data.r#type))
                    })
                    .collect::<Result<Vec<_>, String>>()?;

                union_type.possible_types = possible_types;

                schema.unions.push(union_type);
            }
            Token::Include(path, tokens) => {
                let mut include_schema = generate_schema(tokens)
                    .map_err(|err| format!("Failed to generate schema for {path:?}! Err: {err}"))?;
                schema.append(&mut include_schema);
            }
            Token::Function(name, args, return_type) => {
                let mut function_type = FunctionType::default_with_name(name);

                function_type.args = args
                    .into_iter()
                    .map(|(arg_name, r#type)| {
                        let type_data = lookup_type_data(name, Type::Function, &mut schema, r#type).ok_or(
                        "Failed to lookup type {r#type:?} for arg {arg_name} in function {name}!",
                    )?;
                        Ok(Arg::new(arg_name.to_string(), Some(type_data)))
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                if let Some(return_type) = return_type {
                    function_type.return_type = lookup_type_data(
                        name,
                        Type::Function,
                        &mut schema,
                        return_type,
                    )
                    .ok_or("Failed to lookup type {r#type:?} for return type of function {name}!")?
                    .into();
                }

                schema.functions.push(function_type);
            }
        }
    }

    Ok(schema)
}
