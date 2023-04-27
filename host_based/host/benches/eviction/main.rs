use criterion::measurement::*;
use criterion::*;
use std::time::{Duration, Instant};

pub fn compute_ber(original_data: &[u8], exfiltrated_data: &[u8]) -> f64 {
    let original_data_bits = original_data.len();
    let mut error_bits = 0;

    let missing_bytes = original_data_bits - exfiltrated_data.len();

    for (orig_byte, exfil_byte) in original_data.iter().zip(exfiltrated_data) {
        if orig_byte != exfil_byte {
            error_bits += 1;
        }
    }

    (error_bits + missing_bytes) as f64 / original_data_bits as f64
}

// This function performs the exfiltration and computes the BER
// TODO compile it first, in the bench setup
fn exfiltrate_and_compute_ber(wasmpath: String) -> &'static [u8] {
    // Here we instantiate the scenerio where the exfiltrated data is
    let job = std::thread::spawn(move || {
        host::main::execute_wasm(wasmpath.clone());
    });

    job.join();
    // Get the exfiltration and clean it up
    let exfiltrated_data = unsafe { host::main::exfiltrated.as_slice() };
    unsafe { host::main::exfiltrated.clear() };
    // calculate BER and show result
    let ber = compute_ber("My password".as_bytes(), exfiltrated_data);
    println!("BER: {}", ber);
    exfiltrated_data
}

fn bench_ber(c: &mut Criterion) {
    let mut group = c.benchmark_group("ber");
    let mut all_exfiltrated = vec![];

    group.bench_function(BenchmarkId::new("eviction", 0), |b| {
        b.iter_custom(|iters| {
            // TODO compile the Wasm actors first with different parameters :)
            //
            let mut cumulative_duration = Duration::new(0, 0);
            for _ in 0..iters {
                let start = std::time::Instant::now();
                let exfiltrated_data = black_box(exfiltrate_and_compute_ber(
                    "../rustctre/out/eviction.wasm".to_string(),
                ));
                cumulative_duration += start.elapsed();
                all_exfiltrated.push(exfiltrated_data);
            }

            cumulative_duration
        })
    });

    let bers = all_exfiltrated
        .iter()
        .map(|x| compute_ber("My password".as_bytes(), x))
        .collect::<Vec<f64>>();
    let all_bers = bers.iter().sum::<f64>();
    println!(
        "(ber) {} {:?}",
        all_bers / bers.len() as f64,
        all_exfiltrated
    );
    group.finish();
}

criterion_group!(benches, bench_ber);
criterion_main!(benches);
