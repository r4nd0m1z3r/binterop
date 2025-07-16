use std::{fmt::Debug, mem::ManuallyDrop, str::Utf8Error};

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
impl<T: Debug> Debug for Vector<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.as_slice()).finish()
    }
}
impl<T: Clone> Clone for Vector<T> {
    fn clone(&self) -> Self {
        let mut new = Self::with_capacity(self.capacity);

        let new_slice = new.as_mut_slice();
        new_slice.clone_from_slice(self.as_slice());
        new.length = self.length;

        new
    }
}
impl<T> From<Vec<T>> for Vector<T> {
    fn from(value: Vec<T>) -> Self {
        let (ptr, len, capacity) = {
            let mut vec = ManuallyDrop::new(value);
            (vec.as_mut_ptr(), vec.len(), vec.capacity())
        };

        Self {
            ptr,
            length: len as u64,
            capacity: capacity as u64,
        }
    }
}
impl<T> Into<Vec<T>> for Vector<T> {
    fn into(self) -> Vec<T> {
        unsafe { Vec::from_raw_parts(self.ptr, self.length as usize, self.capacity as usize) }
    }
}
impl<T> Vector<T> {
    pub fn new() -> Self {
        let mut vec = vec![];

        Self {
            ptr: vec.as_mut_ptr(),
            length: vec.len() as u64,
            capacity: vec.capacity() as u64,
        }
    }

    pub unsafe fn offset(&mut self, offset: isize) {
        self.ptr = (self.ptr as isize + offset) as *mut T;
    }

    pub fn with_capacity(capacity: u64) -> Self {
        let mut vec = Vec::with_capacity(capacity as usize);

        Self {
            ptr: vec.as_mut_ptr(),
            length: vec.len() as u64,
            capacity: vec.capacity() as u64,
        }
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.length as usize) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.length as usize) }
    }

    pub fn reserve(&mut self, additional: u64) {
        let mut vec =
            unsafe { Vec::from_raw_parts(self.ptr, self.length as usize, self.capacity as usize) };
        vec.reserve(additional as usize);

        *self = vec.into();
    }

    pub fn push(&mut self, elem: T) {
        let mut vec =
            unsafe { Vec::from_raw_parts(self.ptr, self.length as usize, self.capacity as usize) };
        vec.push(elem);

        *self = vec.into();
    }

    pub fn pop(&mut self) -> Option<T> {
        let mut vec =
            unsafe { Vec::from_raw_parts(self.ptr, self.length as usize, self.capacity as usize) };
        let elem = vec.pop();

        *self = vec.into();

        elem
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct String(pub Vector<u8>);
impl String {
    pub fn as_str(&self) -> Result<&str, Utf8Error> {
        std::str::from_utf8(self.0.as_slice())
    }
}
impl Binterop for String {
    fn binterop_type(_: &mut Schema) -> WrappedType {
        WrappedType::String
    }
}
impl Into<std::string::String> for String {
    fn into(self) -> std::string::String {
        let vec: Vec<u8> = self.0.into();

        std::string::String::from_utf8(vec).unwrap()
    }
}
impl From<std::string::String> for String {
    fn from(value: std::string::String) -> Self {
        let vec = value.into_bytes();

        Self(vec.into())
    }
}
