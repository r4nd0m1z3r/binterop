#![feature(vec_into_raw_parts)]
#![feature(new_uninit)]

use crate::main::helpers::Vector;
use crate::main::{GuestToHost, HostToGuest};
use std::mem;
use std::mem::size_of;

#[path = "../../schemas/main.rs"]
mod main;

#[no_mangle]
pub fn alloc(len: usize) -> *mut u8 {
    let mut buf = Box::<[u8]>::new_uninit_slice(len);
    let ptr = buf.as_mut_ptr();

    mem::forget(buf);

    ptr.cast()
}

#[no_mangle]
pub unsafe fn dealloc(ptr: *mut u8) {
    drop(Box::from_raw(ptr));
}

#[no_mangle]
pub fn process_data(host_to_guest: *const HostToGuest) -> *const GuestToHost {
    let HostToGuest { a, b, c } = unsafe { *host_to_guest };
    let a = a as char;

    println!("Got data from host: (a: {a}, b: {b}, c: {c})");

    let mut msg = Vector::new();
    for &char in format!("Char: {a} | {b} + {c} = {}", b + c)
        .as_bytes()
        .iter()
    {
        msg.push(char);
    }

    let output = alloc(size_of::<GuestToHost>()) as *mut GuestToHost;
    unsafe {
        output.write(GuestToHost { msg });
    }

    output
}
