use binterop::{
    schema::Schema,
    types::{data::DataType, function::FunctionType, r#enum::EnumType, union::UnionType, Type},
};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

pub mod rust;

pub struct SourceFile {
    pub path: PathBuf,
    pub content: String,
}
impl SourceFile {
    pub fn new<T: Into<PathBuf>>(path: T) -> Self {
        Self {
            path: path.into(),
            content: String::new(),
        }
    }

    pub fn contents(mut self, contents: String) -> Self {
        self.content = contents;
        self
    }
}

pub struct LanguageGeneratorState<'a> {
    generated_type_names: HashSet<String>,
    output_files: Vec<SourceFile>,
    file_name: &'a str,
    schema: &'a Schema,
}
impl<'a> LanguageGeneratorState<'a> {
    pub fn new(file_name: &'a str, schema: &'a Schema) -> Self {
        Self {
            generated_type_names: HashSet::new(),
            output_files: Vec::new(),
            file_name,
            schema,
        }
    }
}

impl<'a> LanguageGeneratorState<'a> {
    pub fn is_generated(&self, name: &str) -> bool {
        self.schema
            .type_data_by_name(name)
            .map(|type_data| match type_data.r#type {
                Type::Primitive | Type::Array | Type::Vector | Type::Pointer | Type::String => true,
                Type::Data | Type::Enum | Type::Union | Type::Function => {
                    self.generated_type_names.contains(name)
                }
            })
            .unwrap_or(false)
    }

    pub fn mark_generated(&mut self, name: &str) {
        self.generated_type_names.insert(name.to_string());
    }
}

pub trait LanguageGenerator {
    fn prepare(&mut self, _state: &mut LanguageGeneratorState) -> Result<(), String> {
        Ok(())
    }

    fn generate_from_type_and_index(
        &mut self,
        state: &mut LanguageGeneratorState,
        r#type: Type,
        type_index: usize,
    ) -> Result<(), String> {
        match r#type {
            Type::Data => self.generate_data_type(state, &state.schema.types[type_index]),
            Type::Enum => self.generate_enum_type(state, &state.schema.enums[type_index]),
            Type::Union => self.generate_union_type(state, &state.schema.unions[type_index]),
            Type::Function => {
                self.generate_function_type(state, &state.schema.functions[type_index])
            }
            wrapped_type => Err(format!("Generator should not operate on {wrapped_type:?}")),
        }
    }

    fn generate_data_type(
        &mut self,
        state: &mut LanguageGeneratorState,
        data_type: &DataType,
    ) -> Result<(), String>;

    fn generate_enum_type(
        &mut self,
        state: &mut LanguageGeneratorState,
        enum_type: &EnumType,
    ) -> Result<(), String>;

    fn generate_union_type(
        &mut self,
        state: &mut LanguageGeneratorState,
        union_type: &UnionType,
    ) -> Result<(), String>;

    fn generate_function_type(
        &mut self,
        state: &mut LanguageGeneratorState,
        function_type: &FunctionType,
    ) -> Result<(), String>;

    fn generate(
        &mut self,
        state: &mut LanguageGeneratorState,
        output_dir_path: &Path,
    ) -> Result<(), String> {
        self.prepare(state)?;

        for data_type in &state.schema.types {
            if !state.is_generated(&data_type.name) {
                self.generate_data_type(state, data_type)?;
            }
        }
        for enum_type in &state.schema.enums {
            if !state.is_generated(&enum_type.name) {
                self.generate_enum_type(state, enum_type)?;
            }
        }
        for union_type in &state.schema.unions {
            if !state.is_generated(&union_type.name) {
                self.generate_union_type(state, union_type)?;
            }
        }
        for function_type in &state.schema.functions {
            if !state.is_generated(&function_type.name) {
                self.generate_function_type(state, function_type)?;
            }
        }

        self.finish(state, output_dir_path)?;

        Ok(())
    }

    fn finish(
        &mut self,
        state: &mut LanguageGeneratorState,
        output_dir_path: &Path,
    ) -> Result<(), String> {
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
