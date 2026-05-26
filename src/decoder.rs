//! `Read`-based streaming decoder wrapping `LZ4F_decompress`.
//!
//! [`Decoder`] consumes a single LZ4 frame from an inner [`Read`] source
//! and exposes the decompressed payload as a byte stream. The decoder owns
//! a 4 MiB + 64 KiB scratch buffer used both to stage compressed input and
//! to satisfy `LZ4F_decompress`'s "consume some src, produce some dst"
//! contract. A single [`Decoder`] handles exactly one frame — to decode
//! concatenated frames, instantiate a new decoder per frame (see the CLI
//! in `src/bin/lz4.rs` for an example).
//!
//! The API mirrors the `lz4-rs` crate's [`Decoder`] so that this crate is a
//! drop-in replacement.

use super::liblz4::*;
use super::size_t;
use std::io::{Error, ErrorKind, Read, Result};
use std::ptr;

// 4 MiB block payload + 4-byte block header + 4-byte block checksum +
// frame header (max ~19 bytes) + content checksum trailer (4 bytes) +
// slack so the slice-to-dst fast path in `LZ4F_decompress` can fire even
// for incompressible 4 MiB blocks where the block is exactly the maximum
// size. Without slack the fast path never fires for raw 4 MiB blocks and
// the decoder always pays the buffered-input copy overhead.
const BUFFER_SIZE: usize = 4 * 1024 * 1024 + 64 * 1024;

// NOTE: unsafe to device Clone or Copy, otherwise
// there can be multiple copies of the same inner LZ4 pointer
/// RAII wrapper around an `LZ4F_dctx*` that frees the context on drop.
#[derive(Debug)]
struct DecoderContext {
    c: LZ4FDecompressionContext,
}

// NOTE: unsafe to derive Clone or Copy
/// Streaming LZ4 frame decoder wrapping an inner [`Read`] source.
///
/// Pull bytes via the [`Read`] impl; the decoder transparently reads from
/// the source, feeds compressed input into `LZ4F_decompress`, and emits
/// decompressed bytes into the caller's buffer. When the frame ends,
/// further reads return `Ok(0)`. Call [`finish`](Decoder::finish) to
/// recover the inner reader and confirm the frame was fully consumed.
#[derive(Debug)]
pub struct Decoder<R> {
    c: DecoderContext,
    r: R,
    buf: Box<[u8]>,
    pos: usize,
    len: usize,
    next: usize,
    drain_pending: bool,
}

// No interior mutability, so Decoder is Sync as long as R is Sync.
unsafe impl<R: Read + Sync> Sync for Decoder<R> {}

impl<R: Read> Decoder<R> {
    /// Creates a new decoder which reads its input from the given
    /// input stream. The input stream can be re-acquired by calling
    /// `finish()`
    pub fn new(r: R) -> Result<Decoder<R>> {
        Ok(Decoder {
            r,
            c: DecoderContext::new()?,
            buf: vec![0; BUFFER_SIZE].into_boxed_slice(),
            pos: BUFFER_SIZE,
            len: BUFFER_SIZE,
            // Minimal LZ4 stream size
            next: 11,
            drain_pending: false,
        })
    }

    /// Immutable reader reference.
    pub fn reader(&self) -> &R {
        &self.r
    }

    /// Returns the wrapped reader together with a status result.
    ///
    /// The result is `Ok(())` if the LZ4 frame was fully consumed (the
    /// frame footer and any content checksum were read and accepted) and
    /// `Err(ErrorKind::Interrupted)` if `finish` was called before the end
    /// of the compressed stream.
    pub fn finish(self) -> (R, Result<()>) {
        (
            self.r,
            match self.next {
                0 => Ok(()),
                _ => Err(Error::new(
                    ErrorKind::Interrupted,
                    "Finish runned before read end of compressed stream",
                )),
            },
        )
    }
}

impl<R: Read> Read for Decoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if self.next == 0 || buf.is_empty() {
            return Ok(0);
        }
        let mut dst_offset: usize = 0;
        while dst_offset == 0 {
            if self.pos >= self.len {
                if self.drain_pending {
                    self.len = 0;
                    self.drain_pending = false;
                } else {
                    let need = if self.buf.len() < self.next {
                        self.buf.len()
                    } else {
                        self.next
                    };
                    self.len = self.r.read(&mut self.buf[0..need])?;
                    // NOTE: we do not exit here if there was nothing read
                    // The lz4 context may still have more bytes to emit.
                }
                self.pos = 0;
                self.next -= self.len;
            }
            while (dst_offset < buf.len()) && ((self.pos < self.len) || self.len == 0) {
                let mut src_size = (self.len - self.pos) as size_t;
                let mut dst_size = (buf.len() - dst_offset) as size_t;
                let len = check_error(unsafe {
                    LZ4F_decompress(
                        self.c.c,
                        buf[dst_offset..].as_mut_ptr(),
                        &mut dst_size,
                        self.buf[self.pos..].as_ptr(),
                        &mut src_size,
                        ptr::null(),
                    )
                })?;
                self.pos += src_size as usize;
                dst_offset += dst_size as usize;
                self.drain_pending = len == 1 && dst_size > 0;

                // We need to keep trying to read bytes from the decompressor
                // until it is no longer emitting them, even after it
                // has finished reading bytes.
                if dst_size == 0 && src_size == 0 {
                    if dst_offset > 0 {
                        return Ok(dst_offset);
                    }
                    if self.len == 0 && len != 0 {
                        return Err(Error::new(
                            ErrorKind::UnexpectedEof,
                            "unexpected end of compressed stream",
                        ));
                    }
                    return Ok(dst_offset);
                }

                if len == 0 {
                    self.next = 0;
                    return Ok(dst_offset);
                } else if self.next < len {
                    self.next = len;
                }
            }
        }
        Ok(dst_offset)
    }
}

impl DecoderContext {
    fn new() -> Result<DecoderContext> {
        let mut context = LZ4FDecompressionContext(ptr::null_mut());
        check_error(unsafe { LZ4F_createDecompressionContext(&mut context, LZ4F_VERSION) })?;
        Ok(DecoderContext { c: context })
    }
}

impl Drop for DecoderContext {
    fn drop(&mut self) {
        unsafe { LZ4F_freeDecompressionContext(self.c) };
    }
}

#[cfg(test)]
mod test {
    extern crate rand;

    use self::rand::rngs::StdRng;
    use self::rand::Rng;
    use super::super::encoder::{Encoder, EncoderBuilder};
    use super::Decoder;
    use std::io::{Cursor, Error, ErrorKind, Read, Result, Write};

    const BUFFER_SIZE: usize = 64 * 1024;
    const END_MARK: [u8; 4] = [0x9f, 0x77, 0x22, 0x71];

    struct ErrorWrapper<R: Read, Rn: Rng> {
        r: R,
        rng: Rn,
    }

    impl<R: Read, Rn: Rng> ErrorWrapper<R, Rn> {
        fn new(rng: Rn, read: R) -> Self {
            ErrorWrapper { r: read, rng }
        }
    }

    impl<R: Read, Rn: Rng> Read for ErrorWrapper<R, Rn> {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
            if self.rng.next_u32() & 0x03 == 0 {
                self.r.read(buf)
            } else {
                Err(Error::other("Opss..."))
            }
        }
    }

    struct RetryWrapper<R: Read> {
        r: R,
    }

    impl<R: Read> RetryWrapper<R> {
        fn new(read: R) -> Self {
            RetryWrapper { r: read }
        }
    }

    impl<R: Read> Read for RetryWrapper<R> {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
            loop {
                match self.r.read(buf) {
                    Ok(v) => {
                        return Ok(v);
                    }
                    Err(e) => {
                        if e.kind() == ErrorKind::Other {
                            continue;
                        }
                        return Err(e);
                    }
                }
            }
        }
    }

    fn finish_encode<W: Write>(encoder: Encoder<W>) -> W {
        let (mut buffer, result) = encoder.finish();
        result.unwrap();
        buffer.write_all(&END_MARK).unwrap();
        buffer
    }

    fn finish_decode<R: Read>(decoder: Decoder<R>) {
        let (buffer, result) = decoder.finish();
        result.unwrap();

        let mut mark = Vec::new();
        let mut data = Vec::new();
        mark.write_all(&END_MARK).unwrap();
        RetryWrapper::new(buffer).read_to_end(&mut data).unwrap();
        assert_eq!(mark, data);
    }

    #[test]
    fn test_decoder_empty() {
        let expected: Vec<u8> = Vec::new();
        let buffer = finish_encode(EncoderBuilder::new().level(1).build(Vec::new()).unwrap());

        let mut decoder = Decoder::new(Cursor::new(buffer)).unwrap();
        let mut actual = Vec::new();

        decoder.read_to_end(&mut actual).unwrap();
        assert_eq!(expected, actual);
        finish_decode(decoder);
    }

    #[test]
    fn test_decoder_smallest() {
        let expected: Vec<u8> = Vec::new();
        let mut buffer = b"\x04\x22\x4d\x18\x40\x40\xc0\x00\x00\x00\x00".to_vec();
        buffer.write_all(&END_MARK).unwrap();

        let mut decoder = Decoder::new(Cursor::new(buffer)).unwrap();
        let mut actual = Vec::new();

        decoder.read_to_end(&mut actual).unwrap();
        assert_eq!(expected, actual);
        finish_decode(decoder);
    }

    #[test]
    fn test_decoder_smoke() {
        let mut encoder = EncoderBuilder::new().level(1).build(Vec::new()).unwrap();
        let mut expected = Vec::new();
        expected.write_all(b"Some data").unwrap();
        encoder.write_all(&expected[..4]).unwrap();
        encoder.write_all(&expected[4..]).unwrap();
        let buffer = finish_encode(encoder);

        let mut decoder = Decoder::new(Cursor::new(buffer)).unwrap();
        let mut actual = Vec::new();

        decoder.read_to_end(&mut actual).unwrap();
        assert_eq!(expected, actual);
        finish_decode(decoder);
    }

    #[test]
    fn test_decoder_random() {
        let mut rnd = random();
        let expected = random_stream(&mut rnd, 1027 * 1023 * 7);
        let mut encoder = EncoderBuilder::new().level(1).build(Vec::new()).unwrap();
        encoder.write_all(&expected).unwrap();
        let encoded = finish_encode(encoder);

        let mut decoder = Decoder::new(Cursor::new(encoded)).unwrap();
        let mut actual = Vec::new();
        loop {
            let mut buffer = [0; BUFFER_SIZE];
            let size = decoder.read(&mut buffer).unwrap();
            if size == 0 {
                break;
            }
            actual.write_all(&buffer[0..size]).unwrap();
        }
        assert_eq!(expected, actual);
        finish_decode(decoder);
    }

    #[test]
    fn test_retry_read() {
        let mut rnd = random();
        let expected = random_stream(&mut rnd, 1027 * 1023 * 7);
        let mut encoder = EncoderBuilder::new().level(1).build(Vec::new()).unwrap();
        encoder.write_all(&expected).unwrap();
        let encoded = finish_encode(encoder);

        let mut decoder =
            Decoder::new(ErrorWrapper::new(rnd.clone(), Cursor::new(encoded))).unwrap();
        let mut actual = Vec::new();
        loop {
            let mut buffer = [0; BUFFER_SIZE];
            if let Ok(size) = decoder.read(&mut buffer) {
                if size == 0 {
                    break;
                }
                actual.write_all(&buffer[0..size]).unwrap();
            }
        }

        assert_eq!(expected, actual);
        finish_decode(decoder);
    }

    /// Ensure that we emit the full decompressed stream even if we're
    /// using a very small output buffer.
    #[test]
    fn issue_45() {
        // create an encoder
        let mut enc = crate::EncoderBuilder::new().build(Vec::new()).unwrap();

        // write 'a' 100 times to the encoder
        let text: Vec<u8> = vec![b'a'; 100];
        enc.write_all(&text[..]).unwrap();

        let encoded = finish_encode(enc);

        // read from the decoder, buf_size bytes at a time
        for buf_size in [5, 10, 15, 20, 25] {
            let mut buf = vec![0; buf_size];

            let mut total_bytes_read = 0;

            // create a decoder wrapping the backing buffer
            let mut dec = crate::Decoder::new(&encoded[..]).unwrap();
            while let Ok(n) = dec.read(&mut buf[..]) {
                if n == 0 {
                    break;
                }

                total_bytes_read += n;
            }

            assert_eq!(total_bytes_read, text.len());
        }
    }

    #[test]
    fn truncated_frame_reports_unexpected_eof() {
        let mut encoder = EncoderBuilder::new().level(1).build(Vec::new()).unwrap();
        encoder.write_all(b"truncated frame").unwrap();
        let (encoded, result) = encoder.finish();
        result.unwrap();
        for missing in [1, 4] {
            let mut truncated = encoded.clone();
            truncated.truncate(truncated.len() - missing);

            let mut decoder = Decoder::new(Cursor::new(truncated)).unwrap();
            let mut actual = Vec::new();
            let err = decoder.read_to_end(&mut actual).unwrap_err();
            assert_eq!(err.kind(), ErrorKind::UnexpectedEof);
        }
    }

    fn random() -> StdRng {
        let seed: [u8; 32] = [
            157, 164, 190, 237, 231, 103, 60, 22, 197, 108, 51, 176, 30, 170, 155, 21, 163, 249,
            56, 192, 57, 112, 142, 240, 233, 46, 51, 122, 222, 137, 225, 243,
        ];

        rand::SeedableRng::from_seed(seed)
    }

    fn random_stream<R: Rng>(rng: &mut R, size: usize) -> Vec<u8> {
        (0..size).map(|_| rng.gen()).collect()
    }

    #[test]
    fn test_decoder_send() {
        fn check_send<S: Send>(_: &S) {}
        let dec = Decoder::new(Cursor::new(Vec::new())).unwrap();
        check_send(&dec);
    }
}
