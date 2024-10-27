#![allow(dead_code)]

use std::fmt::Debug;
use std::mem::ManuallyDrop;

#[repr(C)]
pub struct Vector<T> {
    pub ptr: *mut T,
    pub len: u64,
    pub capacity: u64,
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
        new.len = self.len;

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
            len: len as u64,
            capacity: capacity as u64,
        }
    }
}
impl<T> Vector<T> {
    pub fn new() -> Self {
        let mut vec = vec![];

        Self {
            ptr: vec.as_mut_ptr(),
            len: vec.len() as u64,
            capacity: vec.capacity() as u64,
        }
    }

    pub fn with_capacity(capacity: u64) -> Self {
        let mut vec = Vec::with_capacity(capacity as usize);

        Self {
            ptr: vec.as_mut_ptr(),
            len: vec.len() as u64,
            capacity: vec.capacity() as u64,
        }
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len as usize) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len as usize) }
    }

    pub fn reserve(&mut self, additional: u64) {
        let mut vec =
            unsafe { Vec::from_raw_parts(self.ptr, self.len as usize, self.capacity as usize) };
        vec.reserve(additional as usize);

        *self = vec.into();
    }

    pub fn push(&mut self, elem: T) {
        let mut vec =
            unsafe { Vec::from_raw_parts(self.ptr, self.len as usize, self.capacity as usize) };
        vec.push(elem);

        *self = vec.into();
    }

    pub fn pop(&mut self) -> Option<T> {
        let mut vec =
            unsafe { Vec::from_raw_parts(self.ptr, self.len as usize, self.capacity as usize) };
        let elem = vec.pop();

        *self = vec.into();

        elem
    }
}
