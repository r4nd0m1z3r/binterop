use binterop::schema::Schema;

pub mod c_gen;

pub trait LanguageGenerator {
    fn feed(&mut self, schema: &Schema);
    fn output(self) -> String;

    fn output_extension(&self) -> String;
}
