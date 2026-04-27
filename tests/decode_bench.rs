use lz4::sys::{LZ4_compressBound, LZ4_compress_HC, LZ4_compress_default, LZ4_decompress_safe};
use std::os::raw::c_char;
use std::time::Instant;

fn make_source_repeat(target_len: usize) -> Vec<u8> {
    let chunk: Vec<u8> = include_str!("../src/sys.rs").bytes().collect();
    let mut v = Vec::with_capacity(target_len);
    while v.len() < target_len {
        let take = std::cmp::min(chunk.len(), target_len - v.len());
        v.extend_from_slice(&chunk[..take]);
    }
    v
}

fn make_random(target_len: usize) -> Vec<u8> {
    let mut v = Vec::with_capacity(target_len);
    let mut s: u64 = 0xCAFEBABE_DEADBEEF;
    for _ in 0..target_len {
        s = s
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        v.push((s >> 33) as u8);
    }
    v
}

fn compress(input: &[u8], hc: bool) -> Vec<u8> {
    unsafe {
        let bound = LZ4_compressBound(input.len() as i32) as usize;
        let mut out = vec![0u8; bound];
        let written = if hc {
            LZ4_compress_HC(
                input.as_ptr() as *const c_char,
                out.as_mut_ptr() as *mut c_char,
                input.len() as i32,
                out.len() as i32,
                9,
            )
        } else {
            LZ4_compress_default(
                input.as_ptr() as *const c_char,
                out.as_mut_ptr() as *mut c_char,
                input.len() as i32,
                out.len() as i32,
            )
        };
        assert!(written > 0);
        out.truncate(written as usize);
        out
    }
}

fn bench(label: &str, input: &[u8], hc: bool, iters: usize) {
    let compressed = compress(input, hc);
    let mut decoded = vec![0u8; input.len()];

    // Warm + correctness.
    let n = unsafe {
        LZ4_decompress_safe(
            compressed.as_ptr() as *const c_char,
            decoded.as_mut_ptr() as *mut c_char,
            compressed.len() as i32,
            decoded.len() as i32,
        )
    };
    assert_eq!(n as usize, input.len(), "{label}: decoded len");
    assert_eq!(&decoded[..], input, "{label}: decoded bytes");

    let start = Instant::now();
    for _ in 0..iters {
        unsafe {
            LZ4_decompress_safe(
                compressed.as_ptr() as *const c_char,
                decoded.as_mut_ptr() as *mut c_char,
                compressed.len() as i32,
                decoded.len() as i32,
            );
        }
    }
    let elapsed = start.elapsed();
    let total_in = input.len() as f64 * iters as f64;
    let mibps = total_in / elapsed.as_secs_f64() / (1024.0 * 1024.0);
    eprintln!(
        "{label:<40}  iters={iters}  total_time={:?}  per_call={:?}  throughput={mibps:.0} MiB/s  ratio={:.2}",
        elapsed,
        elapsed / iters as u32,
        compressed.len() as f64 / input.len() as f64,
    );
}

#[test]
#[ignore]
fn decode_bench() {
    let src1m = make_source_repeat(1024 * 1024);
    let src4m = make_source_repeat(4 * 1024 * 1024);
    let src16m = make_source_repeat(16 * 1024 * 1024);
    let rnd1m = make_random(1024 * 1024);
    let rnd16m = make_random(16 * 1024 * 1024);

    eprintln!("--- default-level compression, then decode ---");
    bench("source-repeat 1 MiB", &src1m, false, 500);
    bench("source-repeat 4 MiB", &src4m, false, 200);
    bench("source-repeat 16 MiB", &src16m, false, 50);
    bench("random       1 MiB", &rnd1m, false, 500);
    bench("random      16 MiB", &rnd16m, false, 50);

    eprintln!();
    eprintln!("--- HC9 compression, then decode ---");
    bench("source-repeat 4 MiB (hc9)", &src4m, true, 100);
    bench("source-repeat 16 MiB (hc9)", &src16m, true, 30);
}
