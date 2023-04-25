#![feature(asm_experimental_arch)]

const data_size: usize = 11;
static mut public_data: [u8; 160] = [2; 160];

const STRIDE: usize = 512;
const ARRAY_FOR_PREDICTION: [u8; 256 * STRIDE] = [0; 256 * STRIDE];
// If wasm target, move this from here, as well as the victim code

// To avoid optimization of the victim code

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
    let (cache_hit, cache_miss) = reproduction::get_cache_time(&ARRAY_FOR_PREDICTION);
    println!("{} {}", cache_hit, cache_miss);
}
