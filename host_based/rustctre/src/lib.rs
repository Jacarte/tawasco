//! Common code for the cache  time attack
//!
#![feature(asm_experimental_arch)]
use std::arch::asm;
#[cfg(all(target_arch = "x86_64"))]
use std::arch::x86_64::_mm_clflush;
#[cfg(all(target_arch = "x86_64"))]
use std::arch::x86_64::_mm_mfence;

// The comments are used to guide the autmatic estimation script
//////// STRIDE SIZE START
pub const STRIDE: usize = 512;
/////// STRIDE SIZE END

static mut tmp: u8 = 0;

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

// TODO, drop to avoid time in assignment
#[cfg(all(target_arch = "wasm32"))]
#[no_mangle]
#[inline]
pub fn read_memory_offset(ptr: *const u8) -> u8 {
    let mut result = 0u8;
    //println!("Reading from memory");
    unsafe {
        asm!(
            // Push the ptr in the stack
            "local.get {}",
            "i32.load8_u 0",
            "drop",
            //"i32.load8_u",
            //"local.set {}",
            in(local) ptr,
            //lateout(local) result,
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
pub fn get_cache_time(array_for_prediction: &'static [u8; 256 * STRIDE], tries: u64) -> (u64, u64) {
    let mut cache_hit = 0;
    let mut cache_miss = 0;

    // Get the TRIES from env
    for k in 0..10 {
        let j = 1;

        #[cfg(all(target_arch = "x86_64"))]
        unsafe {
            _mm_clflush(&array_for_prediction[j * STRIDE] as *const u8);
        }
        #[cfg(all(target_arch = "wasm32"))]
        unsafe {
            _mm_clflush((j * STRIDE) as *const u8);
        }
        
        // TODO watch out for the optimization of this empty loop            
        unsafe {
            _mm_mfence();
        }
        let addr = &array_for_prediction[j * STRIDE as usize] as *const u8;
        // let start = unsafe { _rdtsc() };
        // Read the mem addr
        let end = unsafe {
            let start = _rdtsc();
            tmp &= read_memory_offset(addr);
            _rdtsc() - start
        };

        cache_miss += end;

        #[cfg(feature = "tracing_hit")]{
            println!("miss.append({})", end);
        }

        for x in 0..tries {
            let end = unsafe {
                let start = _rdtsc();
                tmp &= read_memory_offset(addr);
                _rdtsc() - start
            };
            cache_hit += end;

            #[cfg(feature = "tracing_hit")]{
                println!("hit.append({})", end);
            }
        }

        #[cfg(all(target_arch = "x86_64"))]
        unsafe {
            _mm_clflush(&array_for_prediction[j * STRIDE] as *const u8);
        }
        #[cfg(all(target_arch = "wasm32"))]
        unsafe {
            _mm_clflush((j * STRIDE) as *const u8);
        }

        unsafe {
            _mm_mfence();
        }
    }
    //println!(" >");
    // println!("{}", (90*cache_hit/tries + 10*cache_miss)/100);
    (cache_hit /(10* tries), cache_miss/10 )
}
