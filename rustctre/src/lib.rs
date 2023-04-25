//! Common code for the cache  time attack
//!
#![feature(asm_experimental_arch)]
use std::arch::asm;
#[cfg(all(target_arch = "x86_64"))]
use std::arch::x86_64::_mm_clflush;
#[cfg(all(target_arch = "x86_64"))]
use std::arch::x86_64::_mm_mfence;
#[cfg(all(target_arch = "x86_64"))]
use std::arch::x86_64::_rdtsc;

const STRIDE: usize = 512;

static mut tmp: u8 = 0;

#[cfg(all(target_arch = "x86_64"))]
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

#[cfg(all(target_arch = "wasm32"))]
#[no_mangle]
pub fn read_memory_offset(ptr: *const u8) -> u8 {
    let mut result = 0u8;
    //println!("Reading from memory");
    unsafe {
        asm!(
            // Push the ptr in the stack
            "local.get {}",
            "i32.load8_u 0",
            //"i32.load8_u",
            "local.set {}",
            in(local) ptr,
            lateout(local) result,
            //result = out(reg_byte) _
            options(nostack),

        );
    };
    result
}

// In case of wasm target, then this is imported from the host
// Set the host name as wasi
#[cfg(all(target_arch = "wasm32"))]
extern "C" {
    fn _rdtsc() -> u64;
    fn _mm_clflush(ptr: *const u8);
    fn _mm_mfence();
    fn _mm_lfence();
}

#[no_mangle]
#[allow(dead_code)]
/// Calculates the cache hit and miss time
pub fn get_cache_time(array_for_prediction: &'static [u8; 256 * STRIDE]) -> (u64, u64) {
    let mut cache_hit = 0;
    let mut cache_miss = 0;

    let mut cache_hit_count = 0;
    let mut cache_miss_count = 0;
    // Get the TRIES from env
    let tries = 100;

    for i in 0..tries {
        for i in 0..256 {
            // latencies[i] = 0;
        }
        // PRIME
        for j in 0..256 {
            #[cfg(all(target_arch = "x86_64"))]
            unsafe {
                _mm_clflush(&array_for_prediction[j * STRIDE] as *const u8);
            }
            #[cfg(all(target_arch = "wasm32"))]
            unsafe {
                _mm_clflush((j * STRIDE) as *const u8);
            }

            // for _ in 0..1000 {}
            // TODO watch out for the optimization of this empty loop
            unsafe {
                _mm_mfence();
            }

            for i in 0..20 {
                let addr = &array_for_prediction[j * STRIDE as usize] as *const u8;
                // let start = unsafe { _rdtsc() };
                // Read the mem addr
                let end = unsafe {
                    let start = _rdtsc();
                    tmp &= read_memory_offset(addr);
                    _rdtsc() - start
                };
                // First access is slow
                if i == 0 {
                    cache_miss += end;
                    cache_miss_count += 1;
                } else if i > 3
                /* discard the first 3 calls*/
                {
                    cache_hit += end;
                    cache_hit_count += 1;
                }
            }
        }
    }

    //println!(" >");
    (cache_hit / cache_hit_count, cache_miss / cache_miss_count)
}
