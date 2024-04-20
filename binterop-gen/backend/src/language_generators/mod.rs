use binterop::schema::Schema;
use std::path::Path;

pub mod c_gen;
pub mod rust_gen;

pub trait LanguageGenerator {
    fn feed(&mut self, schema: &Schema) -> Result<(), String>;

    fn write(&self, file_path: &Path) -> Result<(), String>;
}
