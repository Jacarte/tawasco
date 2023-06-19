//! The victim accesses its memory every X time. In between the attacker tries to access the memory
#![feature(asm_experimental_arch)]

use reproduction::*;
use std::arch::asm;
#[cfg(all(target_arch = "x86_64"))]
use std::arch::x86_64::_mm_clflush;
#[cfg(all(target_arch = "x86_64"))]
use std::arch::x86_64::_mm_mfence;
#[cfg(all(target_arch = "x86_64"))]
use crate::_rdtsc;
use std::io::Write;

// In case of wasm target, then this is imported from the host
// Set the host name as wasi
#[cfg(all(target_arch = "wasm32"))]
extern "C" {
    fn _rdtsc() -> u64;
    fn _mm_clflush(ptr: *const u8);
    fn _mm_mfence();
    fn _mm_lfence();
    /// To make the attack easier, we say the code to access which secret index
    fn ping() -> u64;
    fn pong();

    fn save_byte(b: i32);
}

const data_size: usize = 11;
static mut public_data: [u8; 160] = [2; 160];
const array_for_prediction: [u8; 256 * STRIDE] = [0; 256 * STRIDE];
const secret_data: &str = "My password";

// To avoid optimization of the victim code
static mut tmp: u8 = 0;

#[no_mangle]
#[allow(dead_code)]
pub fn main() {
    // This takes at least 5seconds * 1000 = 5000 seconds
    let secret_data_bytes = secret_data.as_bytes();
    loop {        
        let j = unsafe { ping() } as usize;
        // eprintln!("index {}", j);
            
        let t = read_memory_offset(&array_for_prediction[secret_data_bytes[j] as usize * STRIDE] as *const u8);

        if j == 10000 {
            break;
        }     

        unsafe { pong() };
    }
}
