use binterop_macro::binterop_inline;

#[test]
fn inline() {
    binterop_inline! {
    "struct",
    struct SomeOtherType {
        a: f64
    }

    struct Test {
        a: [u8:69],
        b: u32
    }

    struct SomeStruct {
        some_uint: u16,
        some_int: i64,
        some_bool: bool,
        some_float: f32,
        some_pointer: SomeOtherType*,
        some_float_array: [f32:10],
        some_other_type: SomeOtherType,
        some_other_type_array: [SomeOtherType:3],
        some_other_type_vector: <SomeOtherType>,
    }
    }
}
