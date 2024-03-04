use crate::language_generators::LanguageGenerator;
use binterop::schema::{Schema, Type};
use binterop::types::{DataType, EnumType, UnionType};
use case::CaseExt;

#[derive(Default, Debug)]
pub struct CGenerator {
    generated_type_names: Vec<String>,
    output: String,
}
impl CGenerator {
    fn binterop_primitive_name_to_c_primitive_name(name: &str) -> Option<String> {
        match name.chars().next().unwrap() {
            'i' => Some(format!("int{}_t", name.strip_prefix('i').unwrap())),
            'u' => Some(format!("int{}_t", name.strip_prefix('u').unwrap())),
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
        }
    }

    fn generate_data_type(&mut self, schema: &Schema, data_type: &DataType) {
        let mut fields_text = String::new();
        for field in &data_type.fields {
            let type_name = schema.type_name(field.type_index, field.r#type);
            let field_type_name =
                Self::binterop_primitive_name_to_c_primitive_name(&type_name).unwrap_or(type_name);

            fields_text.push_str(&format!("\t{field_type_name} {};\n", field.name));
        }

        self.output
            .push_str("typedef struct __attribute__((packed)) {\n");
        self.output.push_str(&fields_text);
        self.output.push_str(&format!("}} {};\n\n", data_type.name));
    }

    fn generate_enum_type(&mut self, enum_type: &EnumType) {
        let mut variants_text = String::new();
        for variant in &enum_type.variants {
            variants_text.push_str(&format!("\t{variant},\n"));
        }

        self.output.push_str("typedef enum {\n");
        self.output.push_str(&variants_text);
        self.output.push_str(&format!("}} {};\n\n", enum_type.name));
    }

    fn generate_union_type(&mut self, schema: &Schema, union_type: &UnionType) {
        let repr_type_name = schema.type_name(union_type.repr_type_index, Type::Primitive);
        let c_repr_type_name =
            Self::binterop_primitive_name_to_c_primitive_name(&repr_type_name).unwrap();
        let repr_field_text = format!("\t{c_repr_type_name} repr;\n");

        let mut union_text = String::from("\tunion {\n");
        for (variant_type_index, variant_type) in union_type.possible_types.iter().copied() {
            let variant_type_name = schema.type_name(variant_type_index, variant_type);
            let field_name = variant_type_name.to_snake();
            union_text.push_str(&format!("\t\t{variant_type_name} {field_name};\n"));
        }
        union_text.push_str("\t};\n");

        self.output
            .push_str("typedef struct __attribute__((packed)) {\n");
        self.output.push_str(&repr_field_text);
        self.output.push_str(&union_text);
        self.output
            .push_str(&format!("}} {};\n\n", union_type.name));
    }
}
impl LanguageGenerator for CGenerator {
    fn feed(&mut self, schema: &Schema) {
        self.output
            .push_str("#include <stdint.h>\n#include <stdbool.h>\n\n");

        let root_type = &schema.types[schema.root_type_index];
        for field in &root_type.fields {
            let type_name = schema.type_name(field.type_index, field.r#type);
            if self.generated_type_names.contains(&type_name) {
                continue;
            }

            match field.r#type {
                Type::Primitive => {}
                Type::Data => self.generate_data_type(schema, &schema.types[field.type_index]),
                Type::Enum => self.generate_enum_type(&schema.enums[field.type_index]),
                Type::Union => self.generate_union_type(schema, &schema.unions[field.type_index]),
            }
            self.generated_type_names.push(type_name);
        }
        self.generate_data_type(schema, root_type);
    }

    fn output(self) -> String {
        self.output
    }

    fn output_extension(&self) -> String {
        "h".to_string()
    }
}
