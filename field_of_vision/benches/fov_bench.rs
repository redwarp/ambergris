use criterion::{criterion_group, criterion_main, Criterion};
use field_of_vision::FovMap;

pub fn fov_benchmark(c: &mut Criterion) {
    let mut fov = FovMap::new(45, 45);

    c.bench_function("calculate_fov", |bencher| {
        bencher.iter(|| fov.calculate_fov(22, 22, 24));
    });
}

criterion_group!(benches, fov_benchmark);
criterion_main!(benches);
