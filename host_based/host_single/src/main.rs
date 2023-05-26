use std::arch::asm;

use std::io::Write;
use wasmtime::*;
use wasmtime_wasi::sync::WasiCtxBuilder;
use std::mem;
extern crate libc;
use libc::{c_void, size_t};
use std::sync::Mutex;
use std::sync::Arc;
use std::ops::Range;
use std::primitive::char;

static mut NEWMEMCOUNT: u32 = 0;
const PAD: usize = 16;
static mut STATIC_ADDRESS: *mut c_void = 0x2000_0000 as *mut c_void;
static mut STATIC_ADDRESS2: *mut c_void = 0x1000_0000 as *mut c_void;

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
        eprintln!("ty {:?}", ty);
        eprintln!("reserved_size_in_bytes {:?}", reserved_size_in_bytes);
        // * WASM_PAGE_SIZE
        // To shrink the allocated memory for the binary just set the number below to 0.5 for example
        let PSIZE = 1;
        let total_bytes = match maximum {
            Some(max) => max*(PSIZE),
            None => minimum*(PSIZE),
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
    pub fn discard_translations(addr: *mut c_void, len: size_t);
    
    fn callgrind_start();

    fn callgrind_end();

    fn set_lock(val: u8);

    fn create_lock();
}

#[inline]
pub fn read_memory_offset(ptr: *const u8) -> u8 {
    let result: u8 = 0;
    unsafe {
        asm!(
            "mov {result}, [{x}]",
            "mov {result}, [{x}]",
            "mov {result}, [{x}]",
            "mov {result}, [{x}]",
            x = in(reg) ptr,
            result = out(reg_byte) _
        );
    };
    result
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
            rustix::mm::ProtFlags::empty(),
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
    unsafe { eprintln!("allocating at {:?} ({})", STATIC_ADDRESS2, size) };

    let ptr = unsafe {
        let r = rustix::mm::mmap_anonymous(
            STATIC_ADDRESS2,
            size,
            rustix::mm::ProtFlags::READ | rustix::mm::ProtFlags::WRITE,
            rustix::mm::MapFlags::PRIVATE, // rustix::mm::MapFlags::FIXED,
        ).expect("Memory could not be allocated");
        STATIC_ADDRESS2 = STATIC_ADDRESS2.add(size + PAD);
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

/// Creates the WASI support
pub fn create_linker(engine: &wasmtime::Engine) -> wasmtime::Linker<wasmtime_wasi::WasiCtx> {
    let mut linker = wasmtime::Linker::new(&engine);

    wasmtime_wasi::add_to_linker(&mut linker, |s| s).unwrap();
    // These methods are not in WASI by default, yet, let us assume they are
    // It is the same assumption of Swivel
    linker.clone()
}

pub fn execute_wasm(path: String) {
    eprintln!("T-> Executing {}", path);

    let pathcp = path.clone();
    let filename = pathcp.split("/").last().unwrap();
    // let binary = std::fs::read(path).unwrap();
    // The binary was already compiled


    // Compile the binary and execute it with wasmtime

    let mut config = wasmtime::Config::default();
    let allocator = MemoryAllocator;
    let mut config = config.strategy(wasmtime::Strategy::Cranelift);

    // Remove spectre protection    
    let mut config = config.cranelift_nan_canonicalization(false);
    let mut config = unsafe { config.cranelift_flag_set("enable_heap_access_spectre_mitigation", "no") };
    let mut config = unsafe { config.cranelift_flag_set("enable_table_access_spectre_mitigation", "no") }; 
    let mut config = config.with_host_memory(Arc::new(allocator));
    // let mut config = config.parallel_compilation(false);
    let mut config = config.memory_init_cow(true);
    
    // Create a new store
    let engine = wasmtime::Engine::new(&config).unwrap();

    let module = wasmtime::Module::from_file(&engine, pathcp).unwrap(); // unsafe { wasmtime::Module::deserialize_file(&engine, pathcp) }.unwrap(); // wasmtime::Module::new(&engine, binary).unwrap();
    ////
    let args: Vec<String> = std::env::args().collect();

    eprintln!("Wasi context");
    let mut wasi = WasiCtxBuilder::new()
        .inherit_stdio()
        .args( &args[1..])
        .unwrap()
        .build();

    let mut linker = create_linker(&engine);
    let mut store = wasmtime::Store::new(&engine, wasi);
    // Thats it
    store.call_hook(/* when the wasm execution starts, then enable the recording */ |t, tpe|{
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
    });

    eprintln!("Linking module");
    linker.module(&mut store, "", &module).unwrap();



    let func = linker
        .get_default(&mut store, "")
        .unwrap()
        .typed::<(), ()>(&mut store)
        .unwrap();

    for i in 0..1{
        func.call(&mut store, ())
            .unwrap();

    }
    
    unsafe {set_lock(1)};
    
    eprintln!("-> Finished");
}

pub fn main() {
    unsafe { create_lock() };
    // Creates a lock file here
    unsafe {set_lock(1)};

    let args: Vec<String> = std::env::args().collect();

    let path = args.get(1).expect("Pass the wasm file as the first argument");
    execute_wasm(path.clone());

}
