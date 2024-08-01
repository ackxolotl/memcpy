#![allow(dead_code)]
#![allow(unused_macros)]

#![feature(avx512_target_feature)]
#![cfg_attr(target_arch = "aarch64", feature(stdarch_aarch64_prefetch))]
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

#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "avx"))]
unsafe fn memcpy_avx(mut src: *const u8, mut dst: *mut u8, count: usize) {
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
        // we'd probably need multiple loads here to profit from vectorization
        let tmp = vld1q_u8(src);
        vst1q_u8(dst, tmp);
        src = src.add(16);
        _prefetch::<_PREFETCH_READ, _PREFETCH_LOCALITY0>(src as *const i8);
        dst = dst.add(16);
    }
}

unsafe fn create_regions(len: usize) -> (*const u8, *mut u8) {
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

    (src, dst)
}

unsafe fn free_regions(src: *const u8, dst: *mut u8, len: usize) {
    libc::munmap(src as *mut c_void, len);
    libc::munmap(dst as *mut c_void, len);
}

fn criterion_benchmark(c: &mut Criterion) {
    let len = 1 << 28; // 256 MiB to copy

    // source and destination region, source is filled with random data
    let (src, dst) = unsafe { create_regions(len) };

    // benchmarks
    let mut group = c.benchmark_group("memcpy");
    group.bench_function("std", |b| b.iter(|| unsafe { memcpy_std(black_box(src), black_box(dst), black_box(len)) }));
    group.bench_function("loop", |b| b.iter(|| unsafe { memcpy_loop(black_box(src), black_box(dst), black_box(len)) }));
    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "avx"))]
    group.bench_function("avx", |b| b.iter(|| unsafe { memcpy_avx(black_box(src), black_box(dst), black_box(len)) }));
    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "avx512f"))]
    group.bench_function("avx512", |b| b.iter(|| unsafe { memcpy_avx512(black_box(src), black_box(dst), black_box(len)) }));
    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    group.bench_function("neon", |b| b.iter(|| unsafe { memcpy_neon(black_box(src), black_box(dst), black_box(len)) }));
    group.finish();

    // cleanup
    unsafe { free_regions(src, dst, len) };
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

#[cfg(test)]
mod tests {
    use super::*;

    const LEN: usize = 1 << 17; // 128 KiB

    macro_rules! test {
        ($function:tt) => {
            unsafe {
                let (src, dst) = create_regions(LEN); // 128 KiB
                $function(src, dst, LEN);
                assert_eq!(memcmp(src, dst, LEN), 0);
                free_regions(src, dst, LEN);
            }
        };
    }

    #[test]
    fn test_memcpy_std() {
        test!(memcpy_std);
    }

    #[test]
    fn test_memcpy_loop() {
        test!(memcpy_loop);
    }

    #[test]
    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "avx"))]
    fn test_memcpy_avx() {
        test!(memcpy_avx);
    }

    #[test]
    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "avx512f"))]
    fn test_memcpy_avx512() {
        test!(memcpy_avx512);
    }

    #[test]
    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    fn test_memcpy_neon() {
        test!(memcpy_neon);
    }

    unsafe fn memcmp(s1: *const u8, s2: *const u8, len: usize) -> i32 {
        libc::memcmp(s1 as *const c_void, s2 as *const c_void, len as libc::size_t)
    }
}