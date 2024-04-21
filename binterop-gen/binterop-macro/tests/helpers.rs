#![allow(dead_code)]

use std::fmt::Debug;

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
impl<T> Vector<T> {
    pub fn new() -> Self {
        let mut vec = vec![];

        Self {
            ptr: vec.as_mut_ptr(),
            len: vec.len() as u64,
            capacity: vec.capacity() as u64,
        }
    }

    fn update_from_vec(&mut self, vec: Vec<T>) {
        let (ptr, len, capacity) = vec.into_raw_parts();
        self.ptr = ptr;
        self.len = len as u64;
        self.capacity = capacity as u64;
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

        self.update_from_vec(vec);
    }

    pub fn push(&mut self, elem: T) {
        let mut vec =
            unsafe { Vec::from_raw_parts(self.ptr, self.len as usize, self.capacity as usize) };
        vec.push(elem);

        self.update_from_vec(vec);
    }

    pub fn pop(&mut self) -> Option<T> {
        let mut vec =
            unsafe { Vec::from_raw_parts(self.ptr, self.len as usize, self.capacity as usize) };
        let elem = vec.pop();

        self.update_from_vec(vec);

        elem
    }
}
