use criterion::{criterion_group, criterion_main, Criterion};

fn just_pressed() {}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fib 20", |b| b.iter(|| just_pressed()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
