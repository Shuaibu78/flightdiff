//! Throughput benchmark. Speed is the product claim, so it is measured from
//! the first commit rather than asserted in the README.

use criterion::{criterion_group, criterion_main, Criterion};

fn bench_detect(c: &mut Criterion) {
    let bytes = b"ULog\x01\x12\x35".repeat(1);
    c.bench_function("detect_ulog", |b| {
        b.iter(|| vane_core::format::ulog::is_ulog(std::hint::black_box(&bytes)));
    });
}

criterion_group!(benches, bench_detect);
criterion_main!(benches);
