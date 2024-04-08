
#[path = "helpers.rs"]
pub mod helpers;
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct HostToGuest {
    pub a: u8,
    pub b: f64,
    pub c: f64,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct GuestToHost {
    pub msg: helpers::Vector<u8>,
}
