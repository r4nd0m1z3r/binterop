use binterop::schema::Schema;
use std::path::Path;

pub mod c_gen;

pub trait LanguageGenerator {
    fn feed(&mut self, schema: &Schema) -> Result<(), String>;

    fn write(&self, dir_path: &Path) -> Result<(), String>;
}
