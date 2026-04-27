use lz4::sys::{LZ4_compressBound, LZ4_compress_HC, LZ4_decompress_safe};
use std::os::raw::c_char;

fn round_trip(input: &[u8], level: i32) -> Result<(), String> {
    unsafe {
        let bound = LZ4_compressBound(input.len() as i32) as usize;
        let mut compressed = vec![0u8; bound];
        let n = LZ4_compress_HC(
            input.as_ptr() as *const c_char,
            compressed.as_mut_ptr() as *mut c_char,
            input.len() as i32,
            compressed.len() as i32,
            level,
        );
        if n <= 0 {
            return Err(format!("compress returned {n}"));
        }
        let mut decoded = vec![0u8; input.len()];
        let m = LZ4_decompress_safe(
            compressed.as_ptr() as *const c_char,
            decoded.as_mut_ptr() as *mut c_char,
            n,
            decoded.len() as i32,
        );
        if m as usize != input.len() {
            return Err(format!(
                "decompress returned {m}, expected {} (compressed_len={n})",
                input.len()
            ));
        }
        if &decoded[..] != input {
            // find first diff
            for (i, (a, b)) in decoded.iter().zip(input.iter()).enumerate() {
                if a != b {
                    return Err(format!(
                        "byte mismatch at offset {i}: got {a:#x}, expected {b:#x} (compressed_len={n})"
                    ));
                }
            }
            return Err("length matches but bytes differ?".into());
        }
        Ok(())
    }
}

#[test]
fn decode_smoke_4096_counter() {
    let input: Vec<u8> = (0..4096).map(|n| (n & 0xff) as u8).collect();
    for level in 1..=12 {
        round_trip(&input, level).unwrap_or_else(|e| panic!("level {level}: {e}"));
    }
}

#[test]
fn decode_smoke_128k_aaaa() {
    let input: Vec<u8> = vec![b'a'; 128 * 1024];
    for level in 1..=12 {
        round_trip(&input, level).unwrap_or_else(|e| panic!("level {level}: {e}"));
    }
}

#[test]
fn decode_smoke_short_inputs() {
    for n in 0..200 {
        let input: Vec<u8> = (0..n).map(|i| (i * 7) as u8).collect();
        round_trip(&input, 1).unwrap_or_else(|e| panic!("len {n}: {e}"));
    }
}
