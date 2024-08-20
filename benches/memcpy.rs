use criterion::{criterion_group, criterion_main, Criterion};

use std::hint::black_box;

fn criterion_benchmark(c: &mut Criterion) {
    // 256 MiB to copy
    let len = 1 << 28;

    // source and destination region, source is filled with random data
    let (src, dst) = unsafe { black_box(memcpy::create_regions(len)) };

    // benchmarks
    let mut group = c.benchmark_group("memcpy");
    group.bench_function("std", |b| b.iter(|| unsafe { memcpy::memcpy_std(black_box(src), black_box(dst), black_box(len)) }));
    group.bench_function("loop", |b| b.iter(|| unsafe { memcpy::memcpy_loop(black_box(src), black_box(dst), black_box(len)) }));
    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "sse", target_feature = "avx"))]
    group.bench_function("avx", |b| b.iter(|| unsafe { memcpy::memcpy_avx(black_box(src), black_box(dst), black_box(len)) }));
    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "sse", target_feature = "avx512f"))]
    group.bench_function("avx512", |b| b.iter(|| unsafe { memcpy::memcpy_avx512(black_box(src), black_box(dst), black_box(len)) }));
    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    group.bench_function("neon", |b| b.iter(|| unsafe { memcpy::memcpy_neon(black_box(src), black_box(dst), black_box(len)) }));
    group.finish();

    // cleanup
    unsafe { black_box(memcpy::free_regions(src, dst, len)) };
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
