use std::path::PathBuf;

use binterop::{
    schema::Schema,
    types::{data::DataType, function::FunctionType, r#enum::EnumType, union::UnionType, Type},
};
use case::CaseExt;

use crate::language_generators::{LanguageGenerator, LanguageGeneratorState, SourceFile};

#[derive(Default)]
pub struct NimLanguageGenerator {}
impl NimLanguageGenerator {
    fn nim_type_name(r#type: Type, type_index: usize, schema: &Schema) -> String {
        match r#type {
            Type::Array => {
                let array_type = schema.arrays[type_index];
                let inner_type_name =
                    Self::nim_type_name(array_type.inner_type, array_type.inner_type_index, schema);
                format!("array[{}, {inner_type_name}]", array_type.len)
            }
            Type::Vector => {
                let heap_array_type = schema.vectors[type_index];
                let inner_type_name = Self::nim_type_name(
                    heap_array_type.inner_type,
                    heap_array_type.inner_type_index,
                    schema,
                );
                format!("Vector[{inner_type_name}]")
            }
            Type::Pointer => {
                let pointer_type = schema.pointers[type_index];
                let inner_type_name = Self::nim_type_name(
                    pointer_type.inner_type,
                    pointer_type.inner_type_index,
                    schema,
                );

                format!("ptr {inner_type_name}")
            }
            Type::Primitive => {
                let type_name = schema.type_name(r#type, type_index);
                match type_name {
                    type_name if type_name.contains("i") => type_name.replace("i", "int"),
                    type_name if type_name.contains("u") => type_name.replace("u", "uint"),
                    type_name if type_name.contains("f") => type_name.replace("f", "float"),
                    _ => type_name.to_string(),
                }
            }
            _ => schema.type_name(r#type, type_index).to_string(),
        }
    }

    fn output_file_mut<'a>(state: &'a mut LanguageGeneratorState) -> &'a mut SourceFile {
        &mut state.output_files[0]
    }
}

impl LanguageGenerator for NimLanguageGenerator {
    fn prepare(&mut self, state: &mut LanguageGeneratorState) -> Result<(), String> {
        let mut output_file_name = PathBuf::from(state.file_name);
        output_file_name.set_extension("nim");

        let output_file =
            SourceFile::new(output_file_name).contents("import binterop\n\n".to_string());
        state.output_files.push(output_file);

        let std_file =
            SourceFile::new("binterop.nim").contents(include_str!("binterop.nim").to_string());
        state.output_files.push(std_file);

        Ok(())
    }

    fn generate_data_type(
        &mut self,
        state: &mut LanguageGeneratorState,
        data_type: &DataType,
    ) -> Result<(), String> {
        let mut fields_text = String::new();

        for field in &data_type.fields {
            let type_data = state.schema.type_data(field.type_index, field.r#type)?;

            if !state.is_generated(&type_data) {
                self.generate_from_type_and_index(state, field.r#type, field.type_index)?;
            }

            let field_type_name = Self::nim_type_name(field.r#type, field.type_index, state.schema);

            fields_text.push_str(&format!("  {}*: {field_type_name}\n", field.name));
        }

        let output = &mut Self::output_file_mut(state).content;
        output.push_str(&format!(
            "type {}* = object\n{fields_text}\n",
            data_type.name
        ));

        state.mark_generated(&data_type.name);
        Ok(())
    }

    fn generate_enum_type(
        &mut self,
        state: &mut LanguageGeneratorState,
        enum_type: &EnumType,
    ) -> Result<(), String> {
        let mut variants_text = String::new();
        for variant in &enum_type.variants {
            variants_text.push_str(&format!("  {variant}\n"));
        }

        let output = &mut Self::output_file_mut(state).content;
        output.push_str(&format!(
            "type {}* {{.size: sizeof(uint32).}} = enum\n{variants_text}\n",
            enum_type.name
        ));

        state.mark_generated(&enum_type.name);
        Ok(())
    }

    fn generate_union_type(
        &mut self,
        state: &mut LanguageGeneratorState,
        union_type: &UnionType,
    ) -> Result<(), String> {
        let mut enum_type = EnumType::new(&format!("{}Variant", union_type.name), &[]);
        enum_type.variants = union_type
            .possible_types
            .iter()
            .map(|&(type_index, r#type)| {
                format!(
                    "{}Variant",
                    state.schema.type_name(r#type, type_index).to_string()
                )
            })
            .collect();
        self.generate_enum_type(state, &enum_type)?;

        let mut union_fields_text = String::new();
        for (type_index, r#type) in union_type.possible_types.iter().copied() {
            let type_name = Self::nim_type_name(r#type, type_index, state.schema);
            let field_name = type_name.to_camel();
            let value_name = type_name.to_camel_lowercase();

            union_fields_text.push_str(&format!(
                "  of {field_name}Variant:\n    {value_name}*: {type_name}\n",
            ));
        }

        let union_name = union_type.name.to_camel();
        let output = &mut Self::output_file_mut(state).content;
        output.push_str(&format!(
            "type {union_name}* = object\n  case variant: {union_name}Variant\n{union_fields_text}\n\n",
        ));

        Ok(())
    }

    fn generate_function_type(
        &mut self,
        state: &mut LanguageGeneratorState,
        function_type: &FunctionType,
    ) -> Result<(), String> {
        for arg in &function_type.args {
            let type_data = arg.r#type.unwrap();

            if !state.is_generated(&type_data) {
                self.generate_from_type_and_index(state, type_data.r#type, type_data.index)?;
            }
        }

        let args_text = function_type
            .args
            .iter()
            .map(|arg| {
                let type_data = arg.r#type.unwrap();
                let type_name =
                    Self::nim_type_name(type_data.r#type, type_data.index, state.schema);

                format!("{}: {type_name}", arg.name)
            })
            .collect::<Vec<_>>()
            .join(", ");

        let return_type_text = function_type
            .return_type
            .map(|return_type_data| {
                format!(
                    ": {}",
                    Self::nim_type_name(
                        return_type_data.r#type,
                        return_type_data.index,
                        state.schema,
                    )
                )
            })
            .unwrap_or_default();

        let output = &mut Self::output_file_mut(state).content;
        output.push_str(&format!(
            "type {}* = proc({args_text}){return_type_text}\n",
            function_type.name.to_camel_lowercase(),
        ));

        state.mark_generated(&function_type.name);
        Ok(())
    }
}
