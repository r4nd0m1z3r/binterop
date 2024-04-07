#[repr(C)]
pub struct Vector<T> {
    pub ptr: *mut T,
    pub len: u64,
    pub capacity: u64,
}
impl<T> Vector<T> {
    pub fn new() -> Self {
        let vec = vec![];
        let (ptr, len, capacity) = vec.into_raw_parts();

        Self {
            ptr,
            len: len as u64,
            capacity: capacity as u64,
        }
    }
    pub fn as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len as usize) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len as usize) }
    }

    pub fn push(&mut self, elem: T) {
        let mut vec =
            unsafe { Vec::from_raw_parts(self.ptr, self.len as usize, self.capacity as usize) };
        vec.push(elem);

        let (ptr, len, capacity) = vec.into_raw_parts();
        self.ptr = ptr;
        self.len = len as u64;
        self.capacity = capacity as u64;
    }

    pub fn pop(&mut self) -> Option<T> {
        let mut vec =
            unsafe { Vec::from_raw_parts(self.ptr, self.len as usize, self.capacity as usize) };
        let elem = vec.pop();

        let (ptr, len, capacity) = vec.into_raw_parts();
        self.ptr = ptr;
        self.len = len as u64;
        self.capacity = capacity as u64;

        elem
    }
}
