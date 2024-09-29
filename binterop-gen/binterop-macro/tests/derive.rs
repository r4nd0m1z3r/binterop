use binterop_macro::Binterop;

#[repr(C)]
#[derive(Binterop)]
struct Test {
    a: i32,
    b: i32,
    c: Vector<i32>,
}

#[test]
pub fn derive() {
    let mut schema = Schema::default();

    dbg!(Test::binterop_type(&mut schema));
}
