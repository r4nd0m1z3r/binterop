#![feature(vec_into_raw_parts)]

use std::mem::size_of;

use binterop_macro::binterop_inline;

mod helpers;

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

    let some_other_type = SomeOtherType { a: 13.37 };
    let test = Test {
        a: [128; 69],
        b: 1337,
    };

    binterop_inline! {
    "recursive",
    struct Recursive {
        depth: u32,
        recursive: Recursive
    }
    }

    dbg!(some_other_type, test, size_of::<Recursive>());
}
