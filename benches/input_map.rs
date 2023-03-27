use criterion::{criterion_group, criterion_main, Criterion};

fn input_map() {}

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fib 20", |b| b.iter(|| input_map()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
