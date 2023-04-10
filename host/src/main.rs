#[cfg(all(target_arch = "x86_64"))]
use std::arch::x86_64::_mm_clflush;
#[cfg(all(target_arch = "x86_64"))]
use std::arch::x86_64::_mm_lfence;
#[cfg(all(target_arch = "x86_64"))]
use std::arch::x86_64::_mm_mfence;
#[cfg(all(target_arch = "x86_64"))]
use std::arch::x86_64::_rdtsc;

use wasmtime::*;
use wasmtime_wasi::sync::WasiCtxBuilder;

pub fn main() {
    // Load the eviction binary first as the attacker an run it
    // TODO, get the binary from the command line
    let binary = std::fs::read("eviction.wasm").unwrap();

    // Compile the binary and execute it with wasmtime

    // Create a new store
    let engine = wasmtime::Engine::default();
    let module = wasmtime::Module::new(&engine, binary).unwrap();

    let mut linker = wasmtime::Linker::new(&engine);

    wasmtime_wasi::add_to_linker(&mut linker, |s| s).unwrap();
    // These methods are not in WASI by default, yet, let us assume they are
    // It is the same assumption of Swivel
    let mut linker = linker
        .func_wrap(
            "env",
            "_mm_clflush",
            |caller: wasmtime::Caller<'_, _>, param: i32| {
                println!("Got {} from WebAssembly", param);
                // println!("my host state is: {}", caller.data());
            },
        )
        .unwrap();

    let mut linker = linker
        .func_wrap("env", "_mm_mfence", |caller: wasmtime::Caller<'_, _>| {
            println!("_mm_mfence");
        })
        .unwrap();
    let mut linker = linker
        .func_wrap("env", "_rdtsc", |caller: wasmtime::Caller<'_, _>| {
            println!("rdtsc");
            0u64
        })
        .unwrap();

    let mut linker = linker
        .func_wrap(
            "env",
            "_mm_lfence",
            |caller: wasmtime::Caller<'_, _>, param: i32| {
                println!("Got {} from WebAssembly", param);
            },
        )
        .unwrap();
    let wasi = WasiCtxBuilder::new()
        .inherit_stdio()
        .inherit_args()
        .unwrap()
        .build();
    // Here we set the default wasi imports and the artificial ones for timing
    let mut store = wasmtime::Store::new(&engine, wasi);

    // let instance = linker.instantiate(&mut store, &module).unwrap();

    linker.module(&mut store, "", &module).unwrap();
    linker
        .get_default(&mut store, "")
        .unwrap()
        .typed::<(), ()>(&mut store)
        .unwrap()
        .call(&mut store, ())
        .unwrap();
}

