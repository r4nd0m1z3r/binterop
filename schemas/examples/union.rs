
#[path = "helpers.rs"]
pub mod helpers;
#[repr(C)]
pub struct SomeOtherType {
    pub a: f64,
}

#[repr(C)]
pub struct Test {
    pub b: u32,
    pub a: [u8; 69],
}

#[repr(C)]
pub struct SomeStruct {
    pub some_bool: bool,
    pub some_uint: u16,
    pub some_float: f32,
    pub some_int: i64,
    pub some_pointer: *mut SomeOtherType,
    pub some_other_type: SomeOtherType,
    pub some_other_type_array: [SomeOtherType; 3],
    pub some_other_type_vector: helpers::Vector<SomeOtherType>,
    pub some_float_array: [f32; 10],
}

#[repr(C)]
pub enum Color {
    Red,
    Green,
    Blue,
}

#[repr(C)]
pub enum SomeUnionVariant {
    Color,
    SomeStruct,
}

#[repr(C)]
pub union SomeUnionUnion {
    pub color: std::mem::ManuallyDrop<Color>,
    pub some_struct: std::mem::ManuallyDrop<SomeStruct>,
}

#[repr(C)]
pub struct SomeUnion {
    pub variant: SomeUnionVariant,
    pub data: SomeUnionUnion,
}
