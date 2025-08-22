use std::path::PathBuf;

use binterop::{
    schema::Schema,
    types::{function::FunctionType, r#enum::EnumType, union::UnionType, Type},
};
use case::CaseExt;

use crate::language_generators::{LanguageGenerator, SourceFile};

use super::LanguageGeneratorState;

#[derive(Default)]
pub struct GoLanguageGenerator {}
impl GoLanguageGenerator {
    fn go_type_name(r#type: Type, type_index: usize, schema: &Schema) -> String {
        match r#type {
            Type::Primitive => {
                let type_name = schema.type_name(r#type, type_index);
                match type_name {
                    type_name if type_name.contains("i") => type_name.replace("i", "int"),
                    type_name if type_name.contains("u") => type_name.replace("u", "uint"),
                    type_name if type_name.contains("f") => type_name.replace("f", "float"),
                    _ => type_name.to_string(),
                }
            }
            Type::Array => {
                let array_type = schema.arrays[type_index];
                format!(
                    "{}[{}]",
                    GoLanguageGenerator::go_type_name(
                        array_type.inner_type,
                        array_type.inner_type_index,
                        schema
                    ),
                    array_type.len
                )
            }
            Type::Vector => {
                let vector_type = schema.vectors[type_index];
                let inner_type_name = Self::go_type_name(
                    vector_type.inner_type,
                    vector_type.inner_type_index,
                    schema,
                );

                format!("Vector[{inner_type_name}]")
            }
            Type::Pointer => {
                let pointer_type = schema.pointers[type_index];
                let inner_type_name = Self::go_type_name(
                    pointer_type.inner_type,
                    pointer_type.inner_type_index,
                    schema,
                );

                format!("*{}", inner_type_name)
            }
            _ => schema.type_name(r#type, type_index).to_string(),
        }
    }

    fn output_file_mut<'a>(state: &'a mut LanguageGeneratorState) -> &'a mut SourceFile {
        &mut state.output_files[0]
    }
}
impl LanguageGenerator for GoLanguageGenerator {
    fn prepare(&mut self, state: &mut LanguageGeneratorState) -> Result<(), String> {
        let mut output_file_name = PathBuf::from(state.file_name);
        output_file_name.set_extension("go");

        let output_file =
            SourceFile::new(output_file_name).contents("import helpers\n\n".to_string());
        state.output_files.push(output_file);

        Ok(())
    }

    fn generate_data_type(
        &mut self,
        state: &mut LanguageGeneratorState,
        data_type: &binterop::types::data::DataType,
    ) -> Result<(), String> {
        let mut fields_text = String::new();

        for field in &data_type.fields {
            let type_data = state.schema.type_data(field.type_index, field.r#type)?;

            if !state.is_generated(&type_data) {
                self.generate_from_type_and_index(state, field.r#type, field.type_index)?;
            }

            let field_type_name = Self::go_type_name(field.r#type, field.type_index, &state.schema);

            fields_text.push_str(&format!("{} {}\n", field.name, field_type_name));
        }

        let output = &mut Self::output_file_mut(state).content;
        output.push_str(&format!(
            "type {} struct {{\n{fields_text}\n}}\n\n",
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
        output.push_str(&format!("type {} int32\n\n", enum_type.name));

        for (i, variant) in enum_type.variants.iter().enumerate() {
            output.push_str(&format!("const {} = {}\n", variant, i));
        }

        state.mark_generated(&enum_type.name);
        Ok(())
    }

    fn generate_union_type(
        &mut self,
        state: &mut LanguageGeneratorState,
        union_type: &UnionType,
    ) -> Result<(), String> {
        let mut variant_enum = EnumType::new(&format!("{}Variant", union_type.name), &[]);
        variant_enum.variants = union_type
            .possible_types
            .iter()
            .map(|&(type_index, r#type)| {
                format!(
                    "{}Variant",
                    state.schema.type_name(r#type, type_index).to_string()
                )
            })
            .collect();

        self.generate_enum_type(state, &variant_enum)?;

        let variant_size = union_type
            .possible_types
            .iter()
            .map(|&(type_index, r#type)| state.schema.type_size(r#type, type_index).unwrap())
            .max()
            .unwrap();

        let output = &mut Self::output_file_mut(state).content;
        output.push_str(&format!("type {} struct {{\n", union_type.name));

        output.push_str(&format!("  variant {}Variant\n", union_type.name));
        output.push_str(&format!("  data [{variant_size}]byte\n}}\n\n"));

        state.mark_generated(&union_type.name);
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
                    Self::go_type_name(type_data.r#type, type_data.index, state.schema);

                format!("{} {type_name}", arg.name)
            })
            .collect::<Vec<_>>()
            .join(", ");

        let return_type_text = function_type
            .return_type
            .map(|return_type_data| {
                format!(
                    " {}",
                    Self::go_type_name(
                        return_type_data.r#type,
                        return_type_data.index,
                        state.schema,
                    )
                )
            })
            .unwrap_or_default();

        let output = &mut Self::output_file_mut(state).content;
        output.push_str(&format!(
            "type {} func({args_text}){return_type_text}\n",
            function_type.name.to_camel_lowercase(),
        ));

        state.mark_generated(&function_type.name);
        Ok(())
    }
}
