use crate::language_generators::LanguageGenerator;
use binterop::schema::Schema;
use binterop::types::DataType;

#[derive(Default, Debug)]
pub struct CGenerator {
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
}
impl LanguageGenerator for CGenerator {
    fn feed(&mut self, schema: &Schema) {
        self.output
            .push_str("#include <stdint.h>\n#include <stdbool.h>\n\n");
        for data_type in &schema.types {
            self.generate_data_type(schema, data_type);
        }
    }

    fn output(self) -> String {
        self.output
    }

    fn output_extension(&self) -> String {
        "h".to_string()
    }
}
