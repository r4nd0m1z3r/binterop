mod helpers;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Recursive {
	pub recursive: bool,
	pub depth: u32,
}

