use criterion::{black_box, criterion_group, criterion_main, Criterion};
use flat_combining::fc_test;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fc_test", |b| b.iter(|| fc_test()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);