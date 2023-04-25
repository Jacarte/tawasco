use criterion::{
    black_box, criterion_group, criterion_main, measurement::Measurement, BenchmarkGroup,
    BenchmarkId, Criterion, Throughput,
};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq)]
struct BerMeasurement(f64);
struct BerFormatter;

impl criterion::measurement::ValueFormatter for BerFormatter {
    fn format_value(&self, value: f64) -> String {
        // The value will be in nanoseconds so we have to convert to half-seconds.
        format!("{:.6} ber", value)
    }

    fn format_throughput(&self, throughput: &Throughput, value: f64) -> String {
        format!("{:.6} ber", value)
    }

    fn scale_values(&self, ns: f64, values: &mut [f64]) -> &'static str {
        "s"
    }

    fn scale_throughputs(
        &self,
        _typical: f64,
        throughput: &Throughput,
        values: &mut [f64],
    ) -> &'static str {
        "s"
    }

    fn scale_for_machines(&self, values: &mut [f64]) -> &'static str {
        "s"
    }
}

impl Measurement for BerMeasurement {
    type Intermediate = f64;
    type Value = f64;

    fn start(&self) -> Self::Intermediate {
        0.0
    }

    fn end(&self, i: Self::Intermediate) -> Self::Value {
        i
    }
    fn add(&self, v1: &Self::Value, v2: &Self::Value) -> Self::Value {
        *v1 + *v2
    }

    fn zero(&self) -> Self::Value {
        0.0
    }

    fn to_f64(&self, val: &Self::Value) -> f64 {
        *val
    }

    fn formatter(&self) -> &dyn criterion::measurement::ValueFormatter {
        &BerFormatter
    }
}

pub fn compute_ber(original_data: &[u8], exfiltrated_data: &[u8]) -> f64 {
    let original_data_bits = original_data.len() * 8;
    let mut error_bits = 0;

    for (orig_byte, exfil_byte) in original_data.iter().zip(exfiltrated_data) {
        let diff = orig_byte ^ exfil_byte;
        error_bits += diff.count_ones();
    }

    100.0 * error_bits as f64 / original_data_bits as f64
}

// This function performs the exfiltration and computes the BER
fn exfiltrate_and_compute_ber() -> f64 {
    let exfiltrated_data = "My passwor".as_bytes();
    let original_data = "My password".as_bytes(); // Implement this function to provide the original data
    compute_ber(&original_data, &exfiltrated_data)
}

fn bench_ber(c: &mut Criterion) {
    let ber_measurement = BerMeasurement(0.0);
    let mut group = c.benchmark_group("ber");

    //group.measurement(ber_measurement);
    group.bench_function(BenchmarkId::new("ber", 0), |b| {
        b.iter_custom(|iters| {
            let mut total_ber = 0.0;

            for _ in 0..iters {
                total_ber += black_box(exfiltrate_and_compute_ber());
            }

            BerMeasurement(100.0 - total_ber / iters as f64)
        })
    });

    group.finish();
}

criterion_group!(benches, bench_ber);
criterion_main!(benches);
