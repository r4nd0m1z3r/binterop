mod helpers;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct SomeOtherType {
	pub a: f64,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Test {
	pub b: u32,
	pub a: [u8; 69],
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct SomeStruct {
	pub some_bool: bool,
	pub some_uint: u16,
	pub some_float: f32,
	pub some_int: i64,
	pub some_pointer: *mut SomeOtherType,
	pub some_other_type: SomeOtherType,
	pub some_other_type_array: [SomeOtherType; 3],
	pub some_other_type_vector: helpers::Vector<SomeOtherType>,
	pub some_string: String,
	pub some_float_array: [f32; 10],
}

type add = extern "C" fn(a: i32, b: i32) -> f64;
type pass_by_pointer = extern "C" fn(pointer: *mut SomeOtherType);
type vec_sum = extern "C" fn(vec: helpers::Vector<i32>) -> i64;
