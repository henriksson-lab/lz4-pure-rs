use lz4::sys::{
    LZ4_compressBound, LZ4_compress_HC_continue, LZ4_createStreamHC, LZ4_decompress_safe_usingDict,
    LZ4_freeStreamHC, LZ4_loadDictHC,
};
use std::os::raw::c_char;
use std::ptr;
use std::time::Instant;

fn lcg_bytes(len: usize, seed: u64) -> Vec<u8> {
    let mut v = Vec::with_capacity(len);
    let mut s = seed;
    for i in 0..len {
        s = s
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        v.push((s >> 33) as u8 ^ (i & 0x07) as u8);
    }
    v
}

fn compress_with_dict(payload: &[u8], dict: &[u8]) -> Vec<u8> {
    unsafe {
        let bound = LZ4_compressBound(payload.len() as i32) as usize;
        let mut out = vec![0u8; bound];
        let stream = LZ4_createStreamHC();
        assert!(!stream.is_null());
        if !dict.is_empty() {
            LZ4_loadDictHC(stream, dict.as_ptr() as *const c_char, dict.len() as i32);
        }
        let written = LZ4_compress_HC_continue(
            stream,
            payload.as_ptr() as *const c_char,
            out.as_mut_ptr() as *mut c_char,
            payload.len() as i32,
            out.len() as i32,
        );
        LZ4_freeStreamHC(stream);
        assert!(written > 0);
        out.truncate(written as usize);
        out
    }
}

fn bench_case(label: &str, payload_size: usize, dict_size: usize, iters: usize) {
    let dict = lcg_bytes(dict_size, 0xA5A5_5A5A_C0DE_C0DE);
    let payload = lcg_bytes(payload_size, 0xDEAD_BEEF_F00D_F00D);
    let compressed = compress_with_dict(&payload, &dict);
    let mut decoded = vec![0u8; payload.len()];

    // Warm + correctness.
    let dec_len = unsafe {
        LZ4_decompress_safe_usingDict(
            compressed.as_ptr() as *const c_char,
            decoded.as_mut_ptr() as *mut c_char,
            compressed.len() as i32,
            decoded.len() as i32,
            if dict.is_empty() {
                ptr::null()
            } else {
                dict.as_ptr() as *const c_char
            },
            dict.len() as i32,
        )
    };
    assert_eq!(dec_len as usize, payload.len(), "{label}: decoded length");
    assert_eq!(&decoded[..], &payload[..], "{label}: decoded bytes");

    let start = Instant::now();
    for _ in 0..iters {
        unsafe {
            LZ4_decompress_safe_usingDict(
                compressed.as_ptr() as *const c_char,
                decoded.as_mut_ptr() as *mut c_char,
                compressed.len() as i32,
                decoded.len() as i32,
                if dict.is_empty() {
                    ptr::null()
                } else {
                    dict.as_ptr() as *const c_char
                },
                dict.len() as i32,
            );
        }
    }
    let elapsed = start.elapsed();
    let per_call = elapsed / iters as u32;
    let throughput =
        (payload.len() as f64 * iters as f64) / elapsed.as_secs_f64() / (1024.0 * 1024.0);
    eprintln!(
        "{label:<32}  iters={iters}  per_call={per_call:?}  throughput={throughput:.1} MiB/s  compressed={} bytes",
        compressed.len()
    );
}

#[test]
#[ignore]
fn usingdict_bench() {
    // dictSize == 0 — should hit the simple no-dict path.
    bench_case("256 KiB / dict=0", 256 * 1024, 0, 2000);
    // small prefix (< 64KB - 1)
    bench_case("256 KiB / dict=1KiB", 256 * 1024, 1024, 2000);
    bench_case("256 KiB / dict=32KiB", 256 * 1024, 32 * 1024, 2000);
    // full prefix64k
    bench_case("256 KiB / dict=64KiB", 256 * 1024, 64 * 1024, 2000);
    // Larger payloads
    bench_case("1 MiB / dict=64KiB", 1024 * 1024, 64 * 1024, 500);
    bench_case("4 MiB / dict=64KiB", 4 * 1024 * 1024, 64 * 1024, 100);
}
