use crate::language_generators::LanguageGenerator;
use binterop::schema::Schema;
use binterop::types::data::DataType;
use binterop::types::r#enum::EnumType;
use binterop::types::union::UnionType;
use binterop::types::Type;
use case::CaseExt;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub struct RustGenerator {
    generated_type_names: HashSet<String>,
    output: String,
    helpers_output: String,
}
impl Default for RustGenerator {
    fn default() -> Self {
        Self {
            generated_type_names: Default::default(),
            output: "
            #[path = \"helpers.rs\"]
            pub mod helpers;"
                .to_string(),
            helpers_output: "".to_string(),
        }
    }
}
impl RustGenerator {
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
            _ => Ok(()),
        }
    }

    fn generate_data_type(&mut self, schema: &Schema, data_type: &DataType) -> Result<(), String> {
        let mut fields_text = "\n".to_string();
        for field in &data_type.fields {
            let type_name = schema.type_name(field.r#type, field.type_index);
            if !self.generated_type_names.contains(type_name.as_ref()) {
                self.generate_type(schema, field.type_index, field.r#type, Some(&field.name))?;
            }

            let field_type_name = Self::rust_type_name(field.r#type, field.type_index, schema);

            fields_text.push_str(&format!("\tpub {}: {field_type_name},\n", field.name));
        }

        self.output.push_str(&format!(
            "#[repr(C)]
            #[derive(Copy, Clone, Debug)]
            pub struct {} 
            {{{fields_text}}}
            
            ",
            data_type.name
        ));

        self.generated_type_names.insert(data_type.name.clone());

        Ok(())
    }

    fn generate_enum_type(&mut self, enum_type: &EnumType) {
        let mut variants_text = "\n".to_string();
        for variant in &enum_type.variants {
            variants_text.push_str(&format!("\t{variant},\n"));
        }

        self.output.push_str(&format!(
            "#[repr(C)]
            #[derive(Copy, Clone, Debug)]
            pub enum {}
            {{{variants_text}}}
            
            ",
            enum_type.name
        ));

        self.generated_type_names.insert(enum_type.name.clone());
    }

    fn generate_union_type(
        &mut self,
        schema: &Schema,
        union_type: &UnionType,
    ) -> Result<(), String> {
        let mut enum_type = EnumType::new(&format!("{}Variant", union_type.name), &[]);
        enum_type.variants = union_type
            .possible_types
            .iter()
            .map(|&(type_index, r#type)| schema.type_name(r#type, type_index).to_string())
            .collect();
        self.generate_enum_type(&enum_type);

        let mut union_fields_text = String::new();
        for (type_index, r#type) in union_type.possible_types.iter().copied() {
            let type_name = Self::rust_type_name(r#type, type_index, schema);
            let field_name = type_name.to_snake();

            union_fields_text.push_str(&format!(
                "\tpub {field_name}: std::mem::ManuallyDrop<{type_name}>,\n"
            ));
        }

        let union_type_name = &union_type.name;
        self.output.push_str(&format!(
            "
        #[repr(C)]
        pub union {union_type_name}Union {{
            {union_fields_text}
        }}
        "
        ));

        self.output.push_str(&format!(
            "
        #[repr(C)]
        #[derive(Copy, Clone, Debug)]
        pub struct {union_type_name} {{
            pub variant: {union_type_name}Variant,
            pub data: {union_type_name}Union
        }}
        ",
        ));

        Ok(())
    }

    fn generate_helpers(&mut self, _schema: &Schema) {
        self.helpers_output.push_str("#[repr(C)]
            #[derive(Copy, Clone, Debug)]
            pub struct Vector<T> {
                pub ptr: *mut T,
                pub len: u64,
                pub capacity: u64,
            }
            impl<T> Vector<T> {
                pub fn new() -> Self {
                    let mut vec = vec![];

                    Self {
                        ptr: vec.as_mut_ptr(),
                        len: vec.len() as u64,
                        capacity: vec.capacity() as u64,
                    }                
                }
                pub fn as_slice(&self) -> &[T] {
                    unsafe { std::slice::from_raw_parts(self.ptr, self.len as usize) } 
                }

                pub fn as_mut_slice(&mut self) -> &mut [T] {
                    unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len as usize) }
                }

                pub fn push(&mut self, elem: T) {
                    let mut vec = unsafe { Vec::from_raw_parts(self.ptr, self.len as usize, self.capacity as usize) };
                    vec.push(elem);

                    self.ptr = vec.as_mut_ptr();
                    self.len = vec.len() as u64;
                    self.capacity = vec.capacity() as u64;                                    
                }

                pub fn pop(&mut self) -> Option<T> {
                    let mut vec = unsafe { Vec::from_raw_parts(self.ptr, self.len as usize, self.capacity as usize) };
                    let elem = vec.pop();

                    self.ptr = vec.as_mut_ptr();
                    self.len = vec.len() as u64;
                    self.capacity = vec.capacity() as u64;    

                    elem
                }
            }");
    }
}
impl LanguageGenerator for RustGenerator {
    fn feed(&mut self, schema: &Schema) -> Result<(), String> {
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

    fn write(&self, file_path: &Path) -> Result<(), String> {
        let generated_file_path = file_path.with_extension("rs");
        fs::write(&generated_file_path, &self.output)
            .map_err(|err| format!("Failed to write generated language file! Error: {err}"))?;

        let helpers_path = file_path.with_file_name("helpers").with_extension("rs");
        fs::write(&helpers_path, &self.helpers_output)
            .map_err(|err| format!("Failed to write helpers file! Error: {err}"))?;

        match std::process::Command::new("rustfmt")
            .current_dir(file_path.parent().unwrap())
            .arg(generated_file_path)
            .arg(helpers_path)
            .spawn()
        {
            Ok(mut child) => {
                if let Err(err) = child
                    .wait()
                    .map_err(|err| format!("\tFailed to format output files! Err: {err:?}"))
                {
                    println!("\tFailed to format output files! Err: {err:?}");
                }
            }
            Err(err) => {
                println!("\tFailed to format output files! Err: {err:?}");
            }
        }

        Ok(())
    }
}
