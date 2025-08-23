use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

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
                    "[{}]{}",
                    array_type.len,
                    GoLanguageGenerator::go_type_name(
                        array_type.inner_type,
                        array_type.inner_type_index,
                        schema
                    ),
                )
            }
            Type::Vector => {
                let vector_type = schema.vectors[type_index];
                let inner_type_name = Self::go_type_name(
                    vector_type.inner_type,
                    vector_type.inner_type_index,
                    schema,
                );

                format!("binterop.Vector[{inner_type_name}]")
            }
            Type::String => "binterop.String".to_string(),
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

        let package_name = output_file_name.file_stem().unwrap().to_str().unwrap();

        let output_file =
            SourceFile::new(&output_file_name).contents(format!("package {package_name}\n\nimport (\n\t\"binterop/helpers\"\n)\nvar _ = binterop.NewVector[byte]()\n\n"));
        state.output_files.push(output_file);

        let go_mod_file = SourceFile::new("go.mod").contents(format!(
            "module {package_name}\n\ngo 1.24\n\nrequire binterop/helpers v0.0.0\n"
        ));
        state.output_files.push(go_mod_file);

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

            fields_text.push_str(&format!(
                "\t{} {}\n",
                field.name.to_camel(),
                field_type_name
            ));
        }

        let output = &mut Self::output_file_mut(state).content;
        output.push_str(&format!(
            "type {} struct {{\n{fields_text}}}\n\n",
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
        let mut variants_iter = enum_type.variants.iter();
        let first_variant = variants_iter
            .next()
            .ok_or("Enum has no variants")?
            .to_camel();

        let output = &mut Self::output_file_mut(state).content;
        output.push_str(&format!("type {} int32\n", enum_type.name));
        output.push_str(&format!(
            "const (\n\t{first_variant} {} = iota\n",
            &enum_type.name
        ));

        for variant in variants_iter {
            output.push_str(&format!("\t{}\n", variant.to_camel()));
        }

        output.push_str(")\n\n");

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

        output.push_str(&format!("  Variant {}Variant\n", union_type.name));
        output.push_str(&format!("  Data [{variant_size}]byte\n}}\n\n"));

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
                let type_name = Self::go_type_name(type_data.r#type, type_data.index, state.schema);

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
            function_type.name.to_camel(),
        ));

        state.mark_generated(&function_type.name);
        Ok(())
    }

    fn finish(
        &mut self,
        state: &mut LanguageGeneratorState,
        output_dir_path: &Path,
    ) -> Result<(), String> {
        let package_name = Path::new(state.file_name)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap();
        let output_dir_path = output_dir_path.join(package_name);

        let dir_create_result = fs::create_dir(&output_dir_path);
        if let Err(err) = dir_create_result {
            if err.kind() != ErrorKind::AlreadyExists {
                return Err(format!(
                    "Failed to create output directory at {output_dir_path:?}! Err: {err:?}",
                ));
            }
        }

        for output_file in &mut state.output_files {
            let full_path = output_dir_path.join(&output_file.path);

            fs::write(&full_path, &output_file.content).map_err(|err| {
                format!(
                    "Failed to write output file to {}! Err: {err:?}",
                    full_path.display()
                )
            })?;
        }
        Ok(())
    }
}
