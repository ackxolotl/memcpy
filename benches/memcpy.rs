#![feature(avx512_target_feature)]
#![cfg_attr(any(target_arch = "x86", target_arch = "x86_64"), feature(stdarch_x86_avx512))]

use criterion::{criterion_group, criterion_main, Criterion};

use std::ffi::c_void;
use std::fs::File;
use std::hint::black_box;
use std::io::Read;

#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;
#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;

unsafe fn memcpy_std(src: *const u8, dst: *mut u8, count: usize) {
    std::ptr::copy_nonoverlapping(src, dst, count);
}

unsafe fn memcpy_loop(src: *const u8, dst: *mut u8, count: usize) {
    for i in 0..count {
        *dst.add(i) = *src.add(i);
    }
}

#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "avx2"))]
unsafe fn memcpy_avx2(mut src: *const u8, mut dst: *mut u8, count: usize) {
    for _ in 0..(count / 32) {
        // _mm256_stream_load_si256 is missing, sigh
        let tmp = _mm256_load_si256(src as *const __m256i);
        _mm256_stream_si256(dst as *mut __m256i, tmp);
        src = src.add(32);
        _mm_prefetch::<_MM_HINT_T2>(src as *const i8);
        dst = dst.add(32);
    }
}

#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "avx512f"))]
unsafe fn memcpy_avx512(mut src: *const u8, mut dst: *mut u8, count: usize) {
    for _ in 0..(count / 64) {
        // _mm512_stream_load_si512 is missing, sigh
        let tmp = _mm512_load_si512(src as *const i32);
        _mm512_stream_si512(dst as *mut i64, tmp);
        src = src.add(64);
        _mm_prefetch::<_MM_HINT_T2>(src as *const i8);
        dst = dst.add(64);
    }
}

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
unsafe fn memcpy_neon(mut src: *const u8, mut dst: *mut u8, count: usize) {
    for _ in 0..(count / 16) {
        let tmp = vld1q_u8(src);
        vst1q_u8(dst, tmp);
        src = src.add(16);
        dst = dst.add(16);
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let len = 1 << 28; // 256 MiB to copy

    // source region
    let src = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            len,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
            -1,
            0,
        )
    } as *mut u8;

    // destination region
    let dst = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            len,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
            -1,
            0,
        )
    } as *mut u8;

    let mut f = File::open("/dev/urandom").unwrap();
    let s = unsafe { std::slice::from_raw_parts_mut(src, len) };
    f.read_exact(s).unwrap();

    // benchmarks
    let mut group = c.benchmark_group("memcpy");
    group.bench_function("std", |b| b.iter(|| unsafe { memcpy_std(black_box(src), black_box(dst), black_box(len)) }));
    group.bench_function("loop", |b| b.iter(|| unsafe { memcpy_loop(black_box(src), black_box(dst), black_box(len)) }));
    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "avx2"))]
    group.bench_function("avx2", |b| b.iter(|| unsafe { memcpy_avx2(black_box(src), black_box(dst), black_box(len)) }));
    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "avx512f"))]
    group.bench_function("avx512", |b| b.iter(|| unsafe { memcpy_avx512(black_box(src), black_box(dst), black_box(len)) }));
    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    group.bench_function("neon", |b| b.iter(|| unsafe { memcpy_neon(black_box(src), black_box(dst), black_box(len)) }));
    group.finish();

    // cleanup
    unsafe {
        libc::munmap(src as *mut c_void, len);
        libc::munmap(dst as *mut c_void, len);
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
