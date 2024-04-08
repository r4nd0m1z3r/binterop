
#[path = "helpers.rs"]
pub mod helpers;
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub enum Color {
    Red,
    Green,
    Blue,
}
