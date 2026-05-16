//! Pure-Rust translation of the LZ4 compression library, exposing the same
//! public API as the `lz4-rs` crate so it can be used as a drop-in replacement
//! (`use lz4::...`).
//!
//! The crate provides:
//! - [`Encoder`] / [`EncoderBuilder`] and [`Decoder`] — `Write`/`Read` streaming
//!   wrappers around the LZ4 frame format.
//! - [`block`] — a safe block-mode API (`compress` / `decompress`) modelled on
//!   `python-lz4`, with optional uncompressed-size prefix.
//! - [`liblz4`] — error helpers and the re-exported low-level `sys` surface.
//! - [`sys`] — the C-shaped LZ4 block, HC, frame, and streaming functions
//!   translated into Rust.
//!
//! See the crate README for the full design rationale and parity status.
#![doc = include_str!("../README.md")]

pub mod sys;

pub mod liblz4;

mod decoder;
mod encoder;

pub mod block;

pub use crate::decoder::Decoder;
pub use crate::encoder::Encoder;
pub use crate::encoder::EncoderBuilder;
pub use crate::liblz4::version;
pub use crate::liblz4::BlockMode;
pub use crate::liblz4::BlockSize;
pub use crate::liblz4::ContentChecksum;

use crate::sys::{c_char, size_t};
