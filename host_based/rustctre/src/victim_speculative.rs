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

const secret_data: &str = "My password";
const data_size: usize = 16;
/* Set the public data */
const public_data: [u8; 17] = [ 1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16, 83/* S */ ];
const _pad: [u8; 64] = [1; 64];
const array_for_prediction: [u8; 256 * STRIDE] = [0; 256 * STRIDE];

// To avoid optimization of the victim code
static mut tmp: u8 = 0;

#[no_mangle]
#[allow(dead_code)]
pub fn main() {
    // This takes at least 5seconds * 1000 = 5000 seconds
    let secret_data_bytes = &public_data;
    // To force the compiler to add the secret data
    eprintln!("{:?}", secret_data);
    loop {        
        // Will block until the host sends a message
        let j = unsafe { ping() } as usize;
        // The code below is what is speculatively executed
        if j < 16 {
            // Always jump to M
            let t = read_memory_offset(&array_for_prediction[secret_data_bytes[j] as usize * STRIDE] as *const u8);
        }

        if j == 10000 {
            break;
        }     

        unsafe { pong() };
    }
}
