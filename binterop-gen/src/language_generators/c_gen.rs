use crate::language_generators::LanguageGenerator;
use binterop::schema::Schema;
use binterop::types::data::DataType;
use binterop::types::heap_array::HeapArrayType;
use binterop::types::primitives::PRIMITIVES;
use binterop::types::r#enum::EnumType;
use binterop::types::union::UnionType;
use binterop::types::Type;
use case::CaseExt;
use std::collections::HashSet;

#[derive(Default, Debug)]
pub struct CGenerator {
    generated_type_names: HashSet<String>,
    output: String,
}
impl CGenerator {
    fn binterop_primitive_name_to_c_primitive_name(name: &str) -> Option<String> {
        let (name, is_pointer) = if let Some(pointer_inner_type_name) = name.strip_suffix('*') {
            (pointer_inner_type_name, true)
        } else {
            (name, false)
        };

        let mut result = match name.chars().next().unwrap() {
            'i' => Some(format!("int{}_t", name.strip_prefix('i').unwrap())),
            'u' => Some(format!("uint{}_t", name.strip_prefix('u').unwrap())),
            'f' => {
                let bitness = name.strip_prefix('f').unwrap().parse::<u8>().unwrap();
                if bitness == 32 {
                    Some("float".to_string())
                } else if bitness == 64 {
                    Some("double".to_string())
                } else {
                    None
                }
            }
            _ => None,
        };

        if is_pointer {
            if let Some(type_name) = result.as_mut() {
                type_name.push('*')
            }
        }

        result
    }

    fn generate_type(
        &mut self,
        schema: &Schema,
        type_index: usize,
        r#type: Type,
        referer_name: Option<&str>,
    ) -> Result<(), String> {
        let referer_name = referer_name.unwrap_or("Unknown");

        match r#type {
            Type::Primitive | Type::Array | Type::Pointer => Ok(()),
            Type::HeapArray => {
                let heap_array_type = schema.heap_arrays.get(type_index).ok_or(format!(
                    "{referer_name} references vector type which is not present in schema!"
                ))?;
                self.generate_heap_array_type(schema, heap_array_type)
            }
            Type::Data => {
                let data_type = schema.types.get(type_index).ok_or(format!(
                    "{referer_name} references type which is not present in schema!",
                ))?;
                self.generate_data_type(schema, data_type)
            }
            Type::Enum => {
                let enum_type = schema.enums.get(type_index).ok_or(format!(
                    "Variant {} references enum which is not present in schema!",
                    referer_name
                ))?;
                self.generate_enum_type(enum_type);

                Ok(())
            }
            Type::Union => {
                let union_type = schema.unions.get(type_index).ok_or(format!(
                    "Variant {} references union which is not present in schema!",
                    referer_name
                ))?;
                self.generate_union_type(schema, union_type)
            }
        }
    }

    fn generate_heap_array_type(
        &mut self,
        schema: &Schema,
        heap_array_type: &HeapArrayType,
    ) -> Result<(), String> {
        let heap_array_data_type_name = format!(
            "Array{}",
            schema.type_name(heap_array_type.inner_type, heap_array_type.inner_type_index)
        );

        if self
            .generated_type_names
            .contains(&heap_array_data_type_name)
        {
            return Ok(());
        }

        let compatible_pointer_type_index = schema
            .pointers
            .iter()
            .position(|pointer_type| pointer_type.inner_type == heap_array_type.inner_type)
            .ok_or("Failed to find compatible pointer type in schema!")?;
        let pointer_field_data = ("ptr", Type::Pointer, compatible_pointer_type_index);
        let len_field_data = ("len", Type::Primitive, PRIMITIVES.index_of("u64").unwrap());

        let data_type = DataType::new(
            schema,
            &heap_array_data_type_name,
            &[pointer_field_data, len_field_data],
        );
        self.generate_data_type(schema, &data_type)
    }

    fn generate_data_type(&mut self, schema: &Schema, data_type: &DataType) -> Result<(), String> {
        let mut fields_text = String::new();
        for field in &data_type.fields {
            let type_name = schema.type_name(field.r#type, field.type_index);
            if !self.generated_type_names.contains(type_name.as_ref()) {
                self.generate_type(schema, field.type_index, field.r#type, Some(&field.name))?;
            }

            let type_name = schema.type_name(field.r#type, field.type_index);
            let field_type_name =
                Self::binterop_primitive_name_to_c_primitive_name(type_name.as_ref())
                    .unwrap_or_else(|| match field.r#type {
                        Type::Array => {
                            let array_type = schema.arrays[field.type_index];
                            let inner_type_name = schema
                                .type_name(array_type.inner_type, array_type.inner_type_index);

                            if let Some(type_name) =
                                Self::binterop_primitive_name_to_c_primitive_name(
                                    inner_type_name.as_ref(),
                                )
                            {
                                type_name
                            } else {
                                inner_type_name.to_string()
                            }
                        }
                        Type::HeapArray => {
                            let heap_array_type = schema.heap_arrays[field.type_index];
                            format!(
                                "Array{}",
                                schema.type_name(
                                    heap_array_type.inner_type,
                                    heap_array_type.inner_type_index,
                                )
                            )
                        }
                        _ => type_name.to_string(),
                    });

            let field_name = if let Type::Array = field.r#type {
                format!("{}[{}]", field.name, schema.arrays[field.type_index].len)
            } else {
                field.name.clone()
            };
            fields_text.push_str(&format!("\t{field_type_name} {field_name};\n"));
        }

        self.output.push_str(&format!(
            "typedef struct __attribute__((packed)) {{\n{fields_text}}} {};\n\n",
            data_type.name
        ));

        self.generated_type_names.insert(data_type.name.clone());

        Ok(())
    }

    fn generate_enum_type(&mut self, enum_type: &EnumType) {
        let mut variants_text = String::new();
        for variant in &enum_type.variants {
            variants_text.push_str(&format!("\t{variant},\n"));
        }

        self.output.push_str(&format!(
            "typedef enum {{\n{variants_text}}} {};\n\n",
            enum_type.name
        ));

        self.generated_type_names.insert(enum_type.name.clone());
    }

    fn generate_union_type(
        &mut self,
        schema: &Schema,
        union_type: &UnionType,
    ) -> Result<(), String> {
        let repr_type_name = schema.type_name(Type::Primitive, union_type.repr_type_index);
        let c_repr_type_name = Self::binterop_primitive_name_to_c_primitive_name(
            repr_type_name.as_ref(),
        )
        .ok_or(format!(
            "Failed to convert binterop {repr_type_name} primitive to C primitive name!"
        ))?;
        let repr_field_text = format!("\t{c_repr_type_name} repr;\n");

        let mut union_text = String::from("\tunion {\n");
        for (variant_type_index, variant_type) in union_type.possible_types.iter().copied() {
            let type_name = schema.type_name(variant_type, variant_type_index);
            if !self.generated_type_names.contains(type_name.as_ref()) {
                self.generate_type(
                    schema,
                    variant_type_index,
                    variant_type,
                    Some(&union_type.name),
                )?;
            }

            let variant_type_name = schema.type_name(variant_type, variant_type_index);
            let field_name = variant_type_name.to_snake();
            union_text.push_str(&format!("\t\t{variant_type_name} {field_name};\n"));
        }
        union_text.push_str("\t};\n");

        self.output.push_str(&format!(
            "typedef struct __attribute__((packed)) {{\n{repr_field_text}{union_text}}} {};\n\n",
            union_type.name
        ));

        self.generated_type_names.insert(union_type.name.clone());

        Ok(())
    }

    fn generate_helpers(&mut self, schema: &Schema) {
        let mut generated_type_names = HashSet::new();

        for heap_array_type in &schema.heap_arrays {
            let inner_type_name =
                schema.type_name(heap_array_type.inner_type, heap_array_type.inner_type_index);
            let c_inner_type_name =
                Self::binterop_primitive_name_to_c_primitive_name(&inner_type_name)
                    .unwrap_or(inner_type_name.to_string());
            let type_name = format!("Array{inner_type_name}");

            if generated_type_names.contains(&type_name) {
                continue;
            }

            self.output.push_str(&format!(
                "{type_name} {type_name}_new(uint64_t len) {{ return ({type_name}){{ malloc(sizeof({c_inner_type_name}) * len), len }}; }}\n"
            ));

            generated_type_names.insert(type_name);
        }
    }
}
impl LanguageGenerator for CGenerator {
    fn feed(&mut self, schema: &Schema) -> Result<(), String> {
        self.output
            .push_str("#include <stdint.h>\n#include <stdbool.h>\n#include <stdlib.h>\n\n");

        for data_type in &schema.types {
            self.generate_data_type(schema, data_type)?;
        }
        for enum_type in &schema.enums {
            if !self.generated_type_names.contains(&enum_type.name) {
                self.generate_enum_type(enum_type);
            }
        }
        for union_type in &schema.unions {
            if !self.generated_type_names.contains(&union_type.name) {
                self.generate_union_type(schema, union_type)?;
            }
        }
        self.generate_helpers(schema);

        Ok(())
    }

    fn output(self) -> String {
        self.output
    }

    fn output_extension(&self) -> String {
        "h".to_string()
    }
}
