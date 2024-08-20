#![feature(avx512_target_feature)]
#![cfg_attr(target_arch = "aarch64", feature(stdarch_aarch64_prefetch))]
#![cfg_attr(any(target_arch = "x86", target_arch = "x86_64"), feature(stdarch_x86_avx512))]

#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;
#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;

pub unsafe fn memcpy_std(src: *const u8, dst: *mut u8, count: usize) {
    std::ptr::copy_nonoverlapping(src, dst, count);
}

pub unsafe fn memcpy_loop(src: *const u8, dst: *mut u8, count: usize) {
    for i in 0..count {
        *dst.add(i) = *src.add(i);
    }
}

#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "sse", target_feature = "avx"))]
pub unsafe fn memcpy_avx(mut src: *const u8, mut dst: *mut u8, count: usize) {
    for _ in 0..(count / 32) {
        // _mm256_stream_load_si256 is missing, sigh
        let tmp = _mm256_load_si256(src as *const __m256i);
        _mm256_stream_si256(dst as *mut __m256i, tmp);
        src = src.add(32);
        _mm_prefetch::<_MM_HINT_T2>(src as *const i8);
        dst = dst.add(32);
    }
    _mm_sfence();
}

#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "sse", target_feature = "avx512f"))]
pub unsafe fn memcpy_avx512(mut src: *const u8, mut dst: *mut u8, count: usize) {
    for _ in 0..(count / 64) {
        // _mm512_stream_load_si512 is missing, sigh
        let tmp = _mm512_load_si512(src as *const i32);
        _mm512_stream_si512(dst as *mut i64, tmp);
        src = src.add(64);
        _mm_prefetch::<_MM_HINT_T2>(src as *const i8);
        dst = dst.add(64);
    }
    _mm_sfence();
}

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
pub unsafe fn memcpy_neon(mut src: *const u8, mut dst: *mut u8, count: usize) {
    for _ in 0..(count / 16) {
        // we'd probably need multiple loads here to profit from vectorization
        let tmp = vld1q_u8(src);
        vst1q_u8(dst, tmp);
        src = src.add(16);
        _prefetch::<_PREFETCH_READ, _PREFETCH_LOCALITY0>(src as *const i8);
        dst = dst.add(16);
    }
}

pub unsafe fn create_regions(len: usize) -> (*const u8, *mut u8) {
    use std::io::Read;

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

    let mut f = std::fs::File::open("/dev/urandom").unwrap();
    let s = unsafe { std::slice::from_raw_parts_mut(src, len) };
    f.read_exact(s).unwrap();

    (src, dst)
}

pub unsafe fn free_regions(src: *const u8, dst: *mut u8, len: usize) {
    libc::munmap(src as *mut libc::c_void, len);
    libc::munmap(dst as *mut libc::c_void, len);
}

#[cfg(test)]
mod tests {
    use super::*;

    const LEN: usize = 1 << 17; // 128 KiB

    macro_rules! test {
        ($function:tt) => {
            unsafe {
                let (src, dst) = create_regions(LEN);
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
    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "sse", target_feature = "avx"))]
    fn test_memcpy_avx() {
        test!(memcpy_avx);
    }

    #[test]
    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "sse", target_feature = "avx512f"))]
    fn test_memcpy_avx512() {
        test!(memcpy_avx512);
    }

    #[test]
    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    fn test_memcpy_neon() {
        test!(memcpy_neon);
    }

    unsafe fn memcmp(s1: *const u8, s2: *const u8, len: usize) -> i32 {
        libc::memcmp(s1 as *const libc::c_void, s2 as *const libc::c_void, len as libc::size_t)
    }
}
