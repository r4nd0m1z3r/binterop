use std::path::PathBuf;

use binterop::{
    schema::Schema,
    types::{data::DataType, r#enum::EnumType, Type},
};
use case::CaseExt;

use crate::language_generators::{LanguageGenerator, SourceFile};

use super::LanguageGeneratorState;

mod helpers;

#[derive(Default)]
pub struct RustLanguageGenerator {}
impl RustLanguageGenerator {
    fn rust_type_name(r#type: Type, type_index: usize, schema: &Schema) -> String {
        match r#type {
            Type::Array => {
                let array_type = schema.arrays[type_index];
                let inner_type_name =
                    schema.type_name(array_type.inner_type, array_type.inner_type_index);
                format!("[{inner_type_name}; {}]", array_type.len)
            }
            Type::Vector => {
                let heap_array_type = schema.vectors[type_index];
                let inner_type_name =
                    schema.type_name(heap_array_type.inner_type, heap_array_type.inner_type_index);
                format!("helpers::Vector<{inner_type_name}>")
            }
            Type::Pointer => {
                let pointer_type = schema.pointers[type_index];
                let inner_type_name =
                    schema.type_name(pointer_type.inner_type, pointer_type.inner_type_index);

                format!("*mut {inner_type_name}")
            }
            _ => schema.type_name(r#type, type_index).to_string(),
        }
    }

    fn output_file_mut<'a>(state: &'a mut LanguageGeneratorState) -> &'a mut SourceFile {
        &mut state.output_files[0]
    }
}
impl LanguageGenerator for RustLanguageGenerator {
    fn prepare(&mut self, state: &mut LanguageGeneratorState) -> Result<(), String> {
        let mut output_file_name = PathBuf::from(state.file_name);
        output_file_name.set_extension("rs");

        let output_file = SourceFile::new(output_file_name);
        state.output_files.push(output_file);

        let helpers_file =
            SourceFile::new("helpers.rs").contents(include_str!("helpers.rs").to_string());
        state.output_files.push(helpers_file);

        Ok(())
    }

    fn generate_data_type(
        &mut self,
        state: &mut LanguageGeneratorState,
        data_type: &DataType,
    ) -> Result<(), String> {
        let mut fields_text = "\n".to_string();

        for field in &data_type.fields {
            let type_name = state.schema.type_name(field.r#type, field.type_index);
            if !state.is_generated(&type_name) {
                self.generate_from_type_and_index(state, field.r#type, field.type_index)?;
            }

            let field_type_name =
                Self::rust_type_name(field.r#type, field.type_index, state.schema);

            fields_text.push_str(&format!("\tpub {}: {field_type_name},\n", field.name));
        }

        let is_copy = data_type.is_copy(state.schema);
        let output = &mut Self::output_file_mut(state).content;
        output.push_str(&format!(
            "#[repr(C)]
                    #[derive({}Clone, Debug)]
                    pub struct {}
                    {{{fields_text}}}

                    ",
            if is_copy { "Copy, " } else { "" },
            data_type.name
        ));

        state.mark_generated(&data_type.name);
        Ok(())
    }

    fn generate_enum_type(
        &mut self,
        state: &mut LanguageGeneratorState,
        enum_type: &binterop::types::r#enum::EnumType,
    ) -> Result<(), String> {
        let mut variants_text = "\n".to_string();
        for variant in &enum_type.variants {
            variants_text.push_str(&format!("\t{variant},\n"));
        }

        let output = &mut Self::output_file_mut(state).content;
        output.push_str(&format!(
            "#[repr(C)]
                    #[derive(Copy, Clone, Debug)]
                    pub enum {}
                    {{{variants_text}}}

                    ",
            enum_type.name
        ));

        state.mark_generated(&enum_type.name);
        Ok(())
    }

    fn generate_union_type(
        &mut self,
        state: &mut LanguageGeneratorState,
        union_type: &binterop::types::union::UnionType,
    ) -> Result<(), String> {
        let mut enum_type = EnumType::new(&format!("{}Variant", union_type.name), &[]);
        enum_type.variants = union_type
            .possible_types
            .iter()
            .map(|&(type_index, r#type)| state.schema.type_name(r#type, type_index).to_string())
            .collect();
        self.generate_enum_type(state, &enum_type)?;

        let mut union_fields_text = String::new();
        for (type_index, r#type) in union_type.possible_types.iter().copied() {
            let type_name = Self::rust_type_name(r#type, type_index, state.schema);
            let field_name = type_name.to_snake();

            union_fields_text.push_str(&format!(
                "\tpub {field_name}: std::mem::ManuallyDrop<{type_name}>,\n"
            ));
        }

        let is_copy = union_type.is_copy(state.schema);
        let output = &mut Self::output_file_mut(state).content;
        let union_type_name = &union_type.name;
        output.push_str(&format!(
            "
                #[repr(C)]
                pub union {union_type_name}Union {{
                    {union_fields_text}
                }}
                "
        ));

        output.push_str(&format!(
            "
                #[repr(C)]
                #[derive({}Clone, Debug)]
                pub struct {union_type_name} {{
                    pub variant: {union_type_name}Variant,
                    pub data: {union_type_name}Union
                }}
                ",
            if is_copy { "Copy, " } else { "" }
        ));

        Ok(())
    }

    fn generate_function_type(
        &mut self,
        state: &mut LanguageGeneratorState,
        function_type: &binterop::types::function::FunctionType,
    ) -> Result<(), String> {
        for arg in &function_type.args {
            if !state.is_generated(&arg.name) {
                let type_data = arg.r#type.unwrap();
                self.generate_from_type_and_index(state, type_data.r#type, type_data.index)?;
            }
        }

        let args_text = function_type
            .args
            .iter()
            .map(|arg| {
                let type_data = arg.r#type.unwrap();
                (type_data.r#type, type_data.index)
            })
            .map(|(r#type, type_index)| Self::rust_type_name(r#type, type_index, state.schema))
            .collect::<Vec<_>>()
            .join(", ");
        let return_type_data = function_type.return_type.unwrap();
        let return_type_text = Self::rust_type_name(
            return_type_data.r#type,
            return_type_data.index,
            state.schema,
        );

        let output = &mut Self::output_file_mut(state).content;
        output.push_str(&format!(
            "\ntype {} = extern \"C\" fn({args_text}) -> {return_type_text};",
            function_type.name,
        ));

        state.mark_generated(&function_type.name);
        Ok(())
    }
}
