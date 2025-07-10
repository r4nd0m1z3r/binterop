use core::slice;
use std::string::{FromUtf8Error, String as RustString};

use crate::{
    schema::Schema,
    types::{vector::VectorType, WrappedType},
    Binterop,
};

#[repr(C)]
pub struct Vector<T> {
    pub ptr: *mut T,
    pub length: u64,
    pub capacity: u64,
}
impl<T: Binterop> Binterop for Vector<T> {
    fn binterop_type(schema: &mut Schema) -> WrappedType {
        let inner_type = T::binterop_type(schema);
        let inner_type_index = schema
            .wrapped_type_index(&inner_type)
            .expect("Provided schema doesnt contain this type!");

        let vector_type = VectorType::new(inner_type.r#type(), inner_type_index);
        schema.vectors.push(vector_type);

        WrappedType::Vector(vector_type)
    }
}
impl<T> Vector<T> {
    pub fn len(&self) -> usize {
        self.length as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn capacity(&self) -> usize {
        self.capacity as usize
    }
}
impl<T> From<Vec<T>> for Vector<T> {
    fn from(mut value: Vec<T>) -> Self {
        Self {
            ptr: value.as_mut_ptr(),
            length: value.len() as u64,
            capacity: value.capacity() as u64,
        }
    }
}
impl<T> Into<Vec<T>> for Vector<T> {
    fn into(self) -> Vec<T> {
        unsafe { Vec::from_raw_parts(self.ptr, self.length as usize, self.capacity as usize) }
    }
}

#[repr(C)]
pub struct String(Vector<u8>);
impl Binterop for String {
    fn binterop_type(_: &mut Schema) -> WrappedType {
        WrappedType::String
    }
}
impl String {
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.0.ptr, self.0.length as usize) }
    }
}
impl From<RustString> for String {
    fn from(value: RustString) -> Self {
        Self(value.into_bytes().into())
    }
}
impl TryInto<RustString> for String {
    type Error = FromUtf8Error;

    fn try_into(self) -> Result<RustString, Self::Error> {
        let vector: Vec<u8> = self.0.into();
        RustString::from_utf8(vector)
    }
}
