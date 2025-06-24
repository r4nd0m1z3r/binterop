use binterop::schema::Schema;
use binterop::std::Vector;
use binterop::Binterop;
use binterop_macro::Binterop;

#[repr(C)]
#[derive(Binterop)]
struct TestStruct {
    a: i32,
    b: i32,
    c: Vector<i32>,
    d: binterop::std::String,
    e: [u8; 16],
}

#[repr(C)]
#[derive(Binterop)]
enum TestEnum {
    A,
    B,
    C,
}

#[test]
pub fn derive() {
    let mut schema = Schema::default();

    dbg!(TestStruct::binterop_type(&mut schema));
    dbg!(TestEnum::binterop_type(&mut schema));

    let schema_json = backend::helpers::serialize_schema(&schema).unwrap();
    eprintln!("{schema_json}");
}
