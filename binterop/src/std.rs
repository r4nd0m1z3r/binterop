#[repr(C)]
pub struct Vector<T> {
    pub ptr: *mut T,
    pub length: u64,
    pub capacity: u64,
}
