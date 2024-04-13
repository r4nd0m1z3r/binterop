use std::alloc::{Allocator, Global};

#[repr(C)]
pub struct Vector<T, A: Allocator + Clone = Global> {
    ptr: *mut T,
    len: usize,
    cap: usize,
    alloc: A,
}
impl<T, A: Allocator + Clone> Vector<T, A> {
    pub fn new() -> Vector<T, Global> {
        let (ptr, len, cap, alloc) = vec![].into_raw_parts_with_alloc();
        Vector::<T> {
            ptr,
            len,
            cap,
            alloc,
        }
    }

    pub fn new_in(alloc: A) -> Self {
        let (ptr, len, cap, alloc) = Vec::new_in(alloc).into_raw_parts_with_alloc();
        Self {
            ptr,
            len,
            cap,
            alloc,
        }
    }
}
impl<T, A: Allocator + Clone> From<Vec<T, A>> for Vector<T, A> {
    fn from(value: Vec<T, A>) -> Self {
        let (ptr, len, cap, alloc) = value.into_raw_parts_with_alloc();
        Self {
            ptr,
            len,
            cap,
            alloc,
        }
    }
}
impl<T, A: Allocator + Clone> From<Vector<T, A>> for Vec<T, A> {
    fn from(value: Vector<T, A>) -> Self {
        unsafe { Self::from_raw_parts_in(value.ptr, value.len, value.cap, value.alloc.clone()) }
    }
}
impl<T, A: Allocator + Clone> Drop for Vector<T, A> {
    fn drop(&mut self) {
        unsafe {
            drop(Vec::from_raw_parts_in(
                self.ptr,
                self.len,
                self.cap,
                self.alloc.clone(),
            ))
        };
    }
}
