#[cfg(all(target_arch = "x86_64"))]
use std::arch::x86_64::_mm_clflush;
#[cfg(all(target_arch = "x86_64"))]
use std::arch::x86_64::_mm_lfence;
#[cfg(all(target_arch = "x86_64"))]
use std::arch::x86_64::_mm_mfence;
#[cfg(all(target_arch = "x86_64"))]
use std::arch::x86_64::_rdtsc;

use std::arch::asm;
use wasmtime::*;
use wasmtime_wasi::sync::WasiCtxBuilder;

pub fn main() {
    // Load the eviction binary first as the attacker an run it
    // TODO, get the binary from the command line first argument
    let args: Vec<String> = std::env::args().collect();
    let binary = std::fs::read(args.get(1).expect("Please, provide the argument")).unwrap();

    // Compile the binary and execute it with wasmtime

    let mut config = wasmtime::Config::default();
    let config = config.strategy(wasmtime::Strategy::Cranelift);
    // Remove spectre protection
    let config = config.cranelift_nan_canonicalization(false);
    let config = config.memory_init_cow(true);

    // This actually produces the same default binary :|
    // let config = config.cranelift_opt_level(wasmtime::OptLevel::SpeedAndSize);

    // We need to save the generated machine code to disk

    // Create a new store
    let engine = wasmtime::Engine::new(&config).unwrap();

    let module = wasmtime::Module::new(&engine, binary).unwrap();

    // Serialize it
    let serialized = module.serialize().unwrap();
    // Save it to disk
    std::fs::write("module.obj", serialized).unwrap();

    let mut linker = wasmtime::Linker::new(&engine);

    wasmtime_wasi::add_to_linker(&mut linker, |s| s).unwrap();
    // These methods are not in WASI by default, yet, let us assume they are
    // It is the same assumption of Swivel
    let mut linker = linker
        .func_wrap(
            "env",
            "_mm_clflush",
            |mut caller: wasmtime::Caller<'_, _>, param: u32| {
                // get the memory of the module
                let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
                let memory_data = memory.data(&mut caller);
                let addr = &memory_data[param as usize] as *const u8;
                // flush the real address of the memory index
                unsafe {
                    asm! {
                       "clflush [{x}]",
                       x = in(reg) addr
                    }
                }
            },
        )
        .unwrap();

    let mut linker = linker
        .func_wrap(
            "env",
            "_mm_mfence",
            |caller: wasmtime::Caller<'_, _>| unsafe {
                // println!("_mm_mfence");
                _mm_mfence();
            },
        )
        .unwrap();
    let mut linker = linker
        .func_wrap("env", "_rdtsc", |caller: wasmtime::Caller<'_, _>| unsafe {
            _rdtsc()
        })
        .unwrap();

    let mut linker = linker
        .func_wrap(
            "env",
            "_mm_lfence",
            |caller: wasmtime::Caller<'_, _>, param: i32| unsafe {
                _mm_lfence();
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
