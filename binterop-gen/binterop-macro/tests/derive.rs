use std::{alloc::Layout, any::type_name};

use binterop::{
    field::Field,
    schema::Schema,
    std::Vector,
    types::{data::DataType, WrappedType},
    Binterop,
};

#[repr(C)]
struct Test {
    a: i32,
    b: i32,
    c: Vector<i32>,
}

impl Binterop for Test {
    fn binterop_type(schema: &mut Schema) -> WrappedType {
        let mut data_type = DataType::from_fields(
            type_name::<Self>(),
            &[
                Field::new_from_wrapped(&(i32::binterop_type(schema)), schema),
                Field::new_from_wrapped(&(i32::binterop_type(schema)), schema),
                Field::new_from_wrapped(&(Vector::<i32>::binterop_type(schema)), schema),
                Field::new_from_wrapped(&(<[f32; 69]>::binterop_type(schema)), schema),
            ],
        );

        let mut layout = Layout::from_size_align(0, 1).unwrap();
        for field in &mut data_type.fields {
            let field_layout = field.layout(schema);
            let (new_layout, offset) = layout.extend(field_layout).unwrap();
            layout = new_layout;
            field.offset = offset;
        }

        WrappedType::Data(data_type)
    }
}

#[test]
pub fn derive() {
    let mut schema = Schema::default();

    dbg!(Test::binterop_type(&mut schema));
}
