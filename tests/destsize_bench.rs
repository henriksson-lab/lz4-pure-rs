use lz4::sys::{LZ4_compress_destSize, LZ4_decompress_safe};
use std::os::raw::c_char;
use std::time::Instant;

fn make_input(len: usize) -> Vec<u8> {
    // Semi-compressible: a deterministic LCG pseudo-random byte stream with
    // a 16-byte repeating low-bit pattern injected. Compresses to ~70% of
    // input size — enough that destSize truncation actually fires when target
    // is set below that ratio.
    let mut v = Vec::with_capacity(len);
    let mut state: u64 = 0x1234_5678_9abc_def0;
    for i in 0..len {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let r = (state >> 33) as u8;
        let pat = (i & 0x0f) as u8;
        v.push(r ^ pat);
    }
    v
}

fn bench(label: &str, input_size: usize, target_ratio: f64, iters: usize) {
    let input = make_input(input_size);
    let target = (input_size as f64 * target_ratio) as i32;
    let mut dst = vec![0u8; target as usize];

    // Warm up + correctness check (round-trip the truncated output).
    let (warm_consumed, warm_written) = {
        let mut src_size = input.len() as i32;
        let written = unsafe {
            LZ4_compress_destSize(
                input.as_ptr() as *const c_char,
                dst.as_mut_ptr() as *mut c_char,
                &mut src_size,
                target,
            )
        };
        assert!(written > 0, "{label}: compress returned {written}");
        assert!(src_size > 0 && src_size as usize <= input.len());
        assert!(written as usize <= target as usize);
        (src_size as usize, written as usize)
    };
    let mut decoded = vec![0u8; warm_consumed];
    let dec_len = unsafe {
        LZ4_decompress_safe(
            dst.as_ptr() as *const c_char,
            decoded.as_mut_ptr() as *mut c_char,
            warm_written as i32,
            decoded.len() as i32,
        )
    };
    assert_eq!(
        dec_len as usize, warm_consumed,
        "{label}: round-trip length mismatch"
    );
    assert_eq!(
        &decoded[..],
        &input[..warm_consumed],
        "{label}: round-trip data mismatch"
    );

    let start = Instant::now();
    let mut sum_consumed: i64 = 0;
    let mut sum_written: i64 = 0;
    for _ in 0..iters {
        let mut src_size = input.len() as i32;
        let written = unsafe {
            LZ4_compress_destSize(
                input.as_ptr() as *const c_char,
                dst.as_mut_ptr() as *mut c_char,
                &mut src_size,
                target,
            )
        };
        sum_consumed += src_size as i64;
        sum_written += written as i64;
    }
    let elapsed = start.elapsed();
    let per_call = elapsed / iters as u32;
    eprintln!(
        "{label:<24}  iters={iters}  per_call={per_call:?}  consumed/call={}  written/call={}",
        sum_consumed / iters as i64,
        sum_written / iters as i64
    );
}

#[test]
#[ignore]
fn destsize_bench() {
    // Target ratios chosen so that destSize must truncate (input compresses
    // to ~70% of len with this generator, so 0.50 * input forces real work).
    bench("4 KiB / 2 KiB", 4 * 1024, 0.50, 5000);
    bench("64 KiB / 32 KiB", 64 * 1024, 0.50, 1000);
    bench("256 KiB / 128 KiB", 256 * 1024, 0.50, 200);
    bench("1 MiB / 512 KiB", 1024 * 1024, 0.50, 50);
    bench("4 MiB / 2 MiB", 4 * 1024 * 1024, 0.50, 20);
}
