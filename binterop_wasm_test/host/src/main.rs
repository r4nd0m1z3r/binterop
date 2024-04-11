#![feature(vec_into_raw_parts)]

use crate::main::{GuestToHost, HostToGuest};
use std::mem::size_of;
use wasi_common::sync::snapshots::preview_1::add_wasi_snapshot_preview1_to_linker;
use wasi_common::sync::WasiCtxBuilder;
use wasmtime::*;

#[path = "../../schemas/main.rs"]
mod main;

fn main() {
    println!("Hello, world!");

    let engine = Engine::default();
    let mut linker = Linker::new(&engine);
    add_wasi_snapshot_preview1_to_linker(&mut linker, |s| s).unwrap();

    let wasi = WasiCtxBuilder::new().inherit_stdio().build();
    let mut store = Store::new(&engine, wasi);

    let module_path = "binterop_wasm_test/guest/src/guest.wasm";
    let module = Module::from_file(&engine, module_path).unwrap();
    let instance = linker.instantiate(&mut store, &module).unwrap();
    let memory = instance.get_memory(&mut store, "memory").unwrap();
    let memory_ptr = memory.data_ptr(&store);

    let alloc = instance
        .get_func(&mut store, "alloc")
        .unwrap()
        .typed::<u32, u32>(&store)
        .unwrap();
    let dealloc = instance
        .get_func(&mut store, "dealloc")
        .unwrap()
        .typed::<u32, ()>(&store)
        .unwrap();
    let process_data = instance
        .get_func(&mut store, "process_data")
        .unwrap()
        .typed::<u32, u32>(&store)
        .unwrap();

    let input_ptr = alloc
        .call(&mut store, size_of::<HostToGuest>() as u32)
        .unwrap() as usize;

    unsafe {
        memory_ptr
            .add(input_ptr)
            .cast::<HostToGuest>()
            .write(HostToGuest {
                a: b'R',
                b: 13.37,
                c: 4.20,
            });
    }

    let result_ptr = process_data.call(&mut store, input_ptr as u32).unwrap() as usize;
    let mut result = unsafe { memory_ptr.add(result_ptr).cast::<GuestToHost>().read() };
    unsafe {
        result.msg.ptr = memory_ptr.add(result.msg.ptr as usize);
    }

    println!(
        "Got data from guest: (msg: {:?})",
        String::from_utf8_lossy(result.msg.as_slice())
    );

    dealloc.call(&mut store, input_ptr as u32).unwrap();
    dealloc.call(&mut store, result_ptr as u32).unwrap();
}
