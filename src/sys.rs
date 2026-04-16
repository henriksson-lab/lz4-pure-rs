#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub use libc::{c_char, c_int, c_uint, c_ulonglong, c_void, size_t};
use std::cmp;
use std::ptr;
use std::slice;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct LZ4FCompressionContext(pub *mut c_void);
unsafe impl Send for LZ4FCompressionContext {}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct LZ4FDecompressionContext(pub *mut c_void);
unsafe impl Send for LZ4FDecompressionContext {}

pub type LZ4FErrorCode = size_t;

#[derive(Clone, Debug)]
#[repr(u32)]
pub enum BlockSize {
    Default = 0,
    Max64KB = 4,
    Max256KB = 5,
    Max1MB = 6,
    Max4MB = 7,
}

impl BlockSize {
    pub fn get_size(&self) -> usize {
        match self {
            BlockSize::Default | BlockSize::Max64KB => 64 * 1024,
            BlockSize::Max256KB => 256 * 1024,
            BlockSize::Max1MB => 1024 * 1024,
            BlockSize::Max4MB => 4 * 1024 * 1024,
        }
    }
}

#[derive(Clone, Debug)]
#[repr(u32)]
pub enum BlockMode {
    Linked = 0,
    Independent,
}

#[derive(Clone, Debug)]
#[repr(u32)]
pub enum ContentChecksum {
    NoChecksum = 0,
    ChecksumEnabled,
}

#[derive(Clone, Debug)]
#[repr(u32)]
pub enum FrameType {
    Frame = 0,
    SkippableFrame,
}

#[derive(Clone, Debug)]
#[repr(u32)]
pub enum BlockChecksum {
    NoBlockChecksum = 0,
    BlockChecksumEnabled,
}

#[derive(Debug)]
#[repr(C)]
pub struct LZ4FFrameInfo {
    pub block_size_id: BlockSize,
    pub block_mode: BlockMode,
    pub content_checksum_flag: ContentChecksum,
    pub frame_type: FrameType,
    pub content_size: c_ulonglong,
    pub dict_id: c_uint,
    pub block_checksum_flag: BlockChecksum,
}

#[derive(Debug)]
#[repr(C)]
pub struct LZ4FPreferences {
    pub frame_info: LZ4FFrameInfo,
    pub compression_level: c_uint,
    pub auto_flush: c_uint,
    pub favor_dec_speed: c_uint,
    pub reserved: [c_uint; 3],
}

#[derive(Debug)]
#[repr(C)]
pub struct LZ4FCompressOptions {
    pub stable_src: c_uint,
    pub reserved: [c_uint; 3],
}

#[derive(Debug)]
#[repr(C)]
pub struct LZ4FDecompressOptions {
    pub stable_dst: c_uint,
    pub reserved: [c_uint; 3],
}

#[derive(Debug)]
#[repr(C)]
pub struct LZ4StreamEncode {
    _private: [u8; 0],
}

#[derive(Debug)]
#[repr(C)]
pub struct LZ4StreamDecode {
    _private: [u8; 0],
}

#[derive(Debug)]
#[repr(C)]
pub struct LZ4StreamHC {
    _private: [u8; 0],
}

#[derive(Debug)]
#[repr(C)]
pub struct LZ4FCDict {
    _private: [u8; 0],
}

pub const LZ4F_VERSION: c_uint = 100;

const LZ4_VERSION_NUMBER: c_int = 11000;
const MINMATCH: usize = 4;
const HASH_BITS: usize = 16;
const HASH_SIZE: usize = 1 << HASH_BITS;
const LZ4HC_HASH_BITS: usize = 15;
const LZ4HC_HASH_SIZE: usize = 1 << LZ4HC_HASH_BITS;
const LZ4_OPT_NUM: usize = 1 << 12;
const LZ4_DISTANCE_MAX: usize = 64 * 1024 - 1;
const LAST_LITERALS: usize = 5;
const MFLIMIT: usize = 12;
const OPTIMAL_ML: usize = 18;
const LZ4_MAX_INPUT_SIZE: c_int = 0x7E00_0000;
const LZ4HC_CLEVEL_DEFAULT: c_int = 9;
const LZ4HC_CLEVEL_MAX: c_int = 12;
const LZ4F_MAGIC: [u8; 4] = [0x04, 0x22, 0x4D, 0x18];
const LZ4F_SKIPPABLE_MAGIC_MIN: u32 = 0x184D_2A50;
const LZ4F_SKIPPABLE_MAGIC_MAX: u32 = 0x184D_2A5F;
const ERROR_GENERIC: usize = usize::MAX;
const ERROR_DST_TOO_SMALL: usize = usize::MAX - 1;
const ERROR_BAD_HEADER: usize = usize::MAX - 2;
const ERROR_CHECKSUM_INVALID: usize = usize::MAX - 3;

static ERROR_GENERIC_NAME: &[u8] = b"ERROR_GENERIC\0";
static ERROR_DST_NAME: &[u8] = b"ERROR_dstMaxSize_tooSmall\0";
static ERROR_BAD_HEADER_NAME: &[u8] = b"ERROR_frameHeader_incomplete\0";
static ERROR_CHECKSUM_NAME: &[u8] = b"ERROR_contentChecksum_invalid\0";
static ERROR_OK_NAME: &[u8] = b"OK_NoError\0";
static LZ4_VERSION_STRING_BYTES: &[u8] = b"1.10.0\0";

#[derive(Debug)]
struct CompressionCtx {
    prefs: FramePrefs,
    content_hasher: XxHash32,
    dictionary: Vec<u8>,
    external_dictionary: bool,
    started: bool,
}

#[derive(Clone, Copy, Debug)]
struct FramePrefs {
    block_size_id: u8,
    block_independent: bool,
    block_checksum: bool,
    content_checksum: bool,
    content_size: u64,
    compression_level: c_int,
}

impl Default for FramePrefs {
    fn default() -> Self {
        Self {
            block_size_id: 4,
            block_independent: true,
            block_checksum: false,
            content_checksum: false,
            content_size: 0,
            compression_level: 0,
        }
    }
}

#[derive(Debug)]
struct DecompressionCtx {
    input: Vec<u8>,
    pending: Vec<u8>,
    pending_pos: usize,
    pos: usize,
    parsed_header: bool,
    done: bool,
    block_checksum: bool,
    content_checksum: bool,
    content_size: u64,
    content_read: u64,
    block_independent: bool,
    block_max: usize,
    dictionary: Vec<u8>,
    external_dictionary: bool,
    content_hasher: XxHash32,
}

#[derive(Debug)]
struct HcStreamCtx {
    compression_level: c_int,
    dictionary: Vec<u8>,
}

#[derive(Debug, Default)]
struct EncodeStreamCtx {
    dictionary: Vec<u8>,
}

#[derive(Debug, Default)]
struct DecodeStreamCtx {
    dictionary: Vec<u8>,
}

#[derive(Debug, Default)]
struct CDictCtx {
    dictionary: Vec<u8>,
}

impl Default for HcStreamCtx {
    fn default() -> Self {
        Self {
            compression_level: LZ4HC_CLEVEL_DEFAULT,
            dictionary: Vec::new(),
        }
    }
}

impl Default for DecompressionCtx {
    fn default() -> Self {
        Self {
            input: Vec::new(),
            pending: Vec::new(),
            pending_pos: 0,
            pos: 0,
            parsed_header: false,
            done: false,
            block_checksum: false,
            content_checksum: false,
            content_size: 0,
            content_read: 0,
            block_independent: true,
            block_max: 64 * 1024,
            dictionary: Vec::new(),
            external_dictionary: false,
            content_hasher: XxHash32::new(0),
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_versionNumber() -> c_int {
    LZ4_VERSION_NUMBER
}

#[no_mangle]
pub extern "C" fn LZ4_versionString() -> *const c_char {
    LZ4_VERSION_STRING_BYTES.as_ptr() as *const c_char
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_compressBound(size: c_int) -> c_int {
    if size < 0 || size > LZ4_MAX_INPUT_SIZE {
        0
    } else {
        size + (size / 255) + 16
    }
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_compress_default(
    source: *const c_char,
    dest: *mut c_char,
    sourceSize: c_int,
    maxDestSize: c_int,
) -> c_int {
    LZ4_compress_fast(source, dest, sourceSize, maxDestSize, 1)
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_compress(
    source: *const c_char,
    dest: *mut c_char,
    sourceSize: c_int,
) -> c_int {
    LZ4_compress_default(source, dest, sourceSize, LZ4_compressBound(sourceSize))
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_compress_limitedOutput(
    source: *const c_char,
    dest: *mut c_char,
    sourceSize: c_int,
    maxOutputSize: c_int,
) -> c_int {
    LZ4_compress_default(source, dest, sourceSize, maxOutputSize)
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_compress_withState(
    state: *mut c_void,
    source: *const c_char,
    dest: *mut c_char,
    inputSize: c_int,
) -> c_int {
    LZ4_compress_fast_extState(
        state,
        source,
        dest,
        inputSize,
        LZ4_compressBound(inputSize),
        1,
    )
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_compress_limitedOutput_withState(
    state: *mut c_void,
    source: *const c_char,
    dest: *mut c_char,
    inputSize: c_int,
    maxOutputSize: c_int,
) -> c_int {
    LZ4_compress_fast_extState(state, source, dest, inputSize, maxOutputSize, 1)
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_compress_fast(
    source: *const c_char,
    dest: *mut c_char,
    sourceSize: c_int,
    maxDestSize: c_int,
    _acceleration: c_int,
) -> c_int {
    if sourceSize < 0 || maxDestSize <= 0 || source.is_null() || dest.is_null() {
        return 0;
    }
    let src = slice::from_raw_parts(source as *const u8, sourceSize as usize);
    let dst = slice::from_raw_parts_mut(dest as *mut u8, maxDestSize as usize);
    compress_block(src, dst).map_or(0, |n| n as c_int)
}

#[no_mangle]
pub extern "C" fn LZ4_sizeofState() -> c_int {
    cmp::max(std::mem::size_of::<usize>() * HASH_SIZE, 8) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_compress_fast_extState(
    state: *mut c_void,
    source: *const c_char,
    dest: *mut c_char,
    sourceSize: c_int,
    maxDestSize: c_int,
    acceleration: c_int,
) -> c_int {
    if state.is_null() {
        return 0;
    }
    LZ4_compress_fast(source, dest, sourceSize, maxDestSize, acceleration)
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_compress_fast_extState_fastReset(
    state: *mut c_void,
    source: *const c_char,
    dest: *mut c_char,
    sourceSize: c_int,
    maxDestSize: c_int,
    acceleration: c_int,
) -> c_int {
    LZ4_compress_fast_extState(state, source, dest, sourceSize, maxDestSize, acceleration)
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_compress_destSize(
    src: *const c_char,
    dst: *mut c_char,
    srcSizePtr: *mut c_int,
    targetDstSize: c_int,
) -> c_int {
    if src.is_null()
        || dst.is_null()
        || srcSizePtr.is_null()
        || *srcSizePtr < 0
        || targetDstSize <= 0
    {
        return 0;
    }
    let src_slice = slice::from_raw_parts(src as *const u8, *srcSizePtr as usize);
    let dst_slice = slice::from_raw_parts_mut(dst as *mut u8, targetDstSize as usize);
    let Some((consumed, written)) = compress_dest_size(src_slice, dst_slice) else {
        return 0;
    };
    *srcSizePtr = consumed as c_int;
    written as c_int
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_compress_destSize_extState(
    _state: *mut c_void,
    src: *const c_char,
    dst: *mut c_char,
    srcSizePtr: *mut c_int,
    targetDstSize: c_int,
    _acceleration: c_int,
) -> c_int {
    LZ4_compress_destSize(src, dst, srcSizePtr, targetDstSize)
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_compress_HC(
    src: *const c_char,
    dst: *mut c_char,
    srcSize: c_int,
    dstCapacity: c_int,
    compressionLevel: c_int,
) -> c_int {
    if srcSize < 0 || dstCapacity <= 0 || src.is_null() || dst.is_null() {
        return 0;
    }
    let src = slice::from_raw_parts(src as *const u8, srcSize as usize);
    let dst = slice::from_raw_parts_mut(dst as *mut u8, dstCapacity as usize);
    compress_block_hc(src, dst, compressionLevel).map_or(0, |n| n as c_int)
}

#[no_mangle]
pub extern "C" fn LZ4_sizeofStateHC() -> c_int {
    cmp::max(
        std::mem::size_of::<HcStreamCtx>(),
        std::mem::align_of::<HcStreamCtx>(),
    ) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_compress_HC_extStateHC(
    stateHC: *mut c_void,
    src: *const c_char,
    dst: *mut c_char,
    srcSize: c_int,
    maxDstSize: c_int,
    compressionLevel: c_int,
) -> c_int {
    if stateHC.is_null() {
        return 0;
    }
    LZ4_compress_HC(src, dst, srcSize, maxDstSize, compressionLevel)
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_compress_HC_extStateHC_fastReset(
    state: *mut c_void,
    src: *const c_char,
    dst: *mut c_char,
    srcSize: c_int,
    dstCapacity: c_int,
    compressionLevel: c_int,
) -> c_int {
    LZ4_compress_HC_extStateHC(state, src, dst, srcSize, dstCapacity, compressionLevel)
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_compress_HC_destSize(
    stateHC: *mut c_void,
    src: *const c_char,
    dst: *mut c_char,
    srcSizePtr: *mut c_int,
    targetDstSize: c_int,
    compressionLevel: c_int,
) -> c_int {
    if stateHC.is_null()
        || src.is_null()
        || dst.is_null()
        || srcSizePtr.is_null()
        || *srcSizePtr < 0
        || targetDstSize <= 0
    {
        return 0;
    }

    let src_len = *srcSizePtr as usize;
    let src_slice = slice::from_raw_parts(src as *const u8, src_len);
    let dst_slice = slice::from_raw_parts_mut(dst as *mut u8, targetDstSize as usize);
    let Some((consumed, written)) = compress_hc_dest_size(src_slice, dst_slice, compressionLevel)
    else {
        return 0;
    };
    *srcSizePtr = consumed as c_int;
    written as c_int
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_compressHC(
    src: *const c_char,
    dst: *mut c_char,
    srcSize: c_int,
) -> c_int {
    LZ4_compress_HC(src, dst, srcSize, LZ4_compressBound(srcSize), 0)
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_compressHC_limitedOutput(
    src: *const c_char,
    dst: *mut c_char,
    srcSize: c_int,
    maxDstSize: c_int,
) -> c_int {
    LZ4_compress_HC(src, dst, srcSize, maxDstSize, 0)
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_compressHC2(
    src: *const c_char,
    dst: *mut c_char,
    srcSize: c_int,
    cLevel: c_int,
) -> c_int {
    LZ4_compress_HC(src, dst, srcSize, LZ4_compressBound(srcSize), cLevel)
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_compressHC2_limitedOutput(
    src: *const c_char,
    dst: *mut c_char,
    srcSize: c_int,
    maxDstSize: c_int,
    cLevel: c_int,
) -> c_int {
    LZ4_compress_HC(src, dst, srcSize, maxDstSize, cLevel)
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_compressHC_withStateHC(
    state: *mut c_void,
    src: *const c_char,
    dst: *mut c_char,
    srcSize: c_int,
) -> c_int {
    LZ4_compress_HC_extStateHC(state, src, dst, srcSize, LZ4_compressBound(srcSize), 0)
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_compressHC_limitedOutput_withStateHC(
    state: *mut c_void,
    src: *const c_char,
    dst: *mut c_char,
    srcSize: c_int,
    maxDstSize: c_int,
) -> c_int {
    LZ4_compress_HC_extStateHC(state, src, dst, srcSize, maxDstSize, 0)
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_compressHC2_withStateHC(
    state: *mut c_void,
    src: *const c_char,
    dst: *mut c_char,
    srcSize: c_int,
    cLevel: c_int,
) -> c_int {
    LZ4_compress_HC_extStateHC(state, src, dst, srcSize, LZ4_compressBound(srcSize), cLevel)
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_compressHC2_limitedOutput_withStateHC(
    state: *mut c_void,
    src: *const c_char,
    dst: *mut c_char,
    srcSize: c_int,
    maxDstSize: c_int,
    cLevel: c_int,
) -> c_int {
    LZ4_compress_HC_extStateHC(state, src, dst, srcSize, maxDstSize, cLevel)
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_decompress_safe(
    source: *const c_char,
    dest: *mut c_char,
    compressedSize: c_int,
    maxDecompressedSize: c_int,
) -> c_int {
    if compressedSize < 0 || maxDecompressedSize < 0 || source.is_null() || dest.is_null() {
        return -1;
    }
    let src = slice::from_raw_parts(source as *const u8, compressedSize as usize);
    let dst = slice::from_raw_parts_mut(dest as *mut u8, maxDecompressedSize as usize);
    decompress_block(src, dst).map_or(-1, |n| n as c_int)
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_decompress_safe_usingDict(
    source: *const c_char,
    dest: *mut c_char,
    compressedSize: c_int,
    maxDecompressedSize: c_int,
    dictStart: *const c_char,
    dictSize: c_int,
) -> c_int {
    if compressedSize < 0
        || maxDecompressedSize < 0
        || dictSize < 0
        || source.is_null()
        || dest.is_null()
        || (dictSize > 0 && dictStart.is_null())
    {
        return -1;
    }
    let src = slice::from_raw_parts(source as *const u8, compressedSize as usize);
    let dst = slice::from_raw_parts_mut(dest as *mut u8, maxDecompressedSize as usize);
    let dict = if dictSize > 0 {
        slice::from_raw_parts(dictStart as *const u8, dictSize as usize)
    } else {
        &[]
    };
    decompress_block_with_dict(src, dst, dict).map_or(-1, |n| n as c_int)
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_decompress_safe_partial(
    source: *const c_char,
    dest: *mut c_char,
    compressedSize: c_int,
    targetOutputSize: c_int,
    dstCapacity: c_int,
) -> c_int {
    if compressedSize < 0
        || targetOutputSize < 0
        || dstCapacity < 0
        || targetOutputSize > dstCapacity
        || source.is_null()
        || dest.is_null()
    {
        return -1;
    }
    let src = slice::from_raw_parts(source as *const u8, compressedSize as usize);
    let dst = slice::from_raw_parts_mut(dest as *mut u8, dstCapacity as usize);
    decompress_block_partial(src, dst, targetOutputSize as usize).map_or(-1, |n| n as c_int)
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_decompress_safe_partial_usingDict(
    source: *const c_char,
    dest: *mut c_char,
    compressedSize: c_int,
    targetOutputSize: c_int,
    dstCapacity: c_int,
    dictStart: *const c_char,
    dictSize: c_int,
) -> c_int {
    if compressedSize < 0
        || targetOutputSize < 0
        || dstCapacity < 0
        || dictSize < 0
        || targetOutputSize > dstCapacity
        || source.is_null()
        || dest.is_null()
        || (dictSize > 0 && dictStart.is_null())
    {
        return -1;
    }
    let src = slice::from_raw_parts(source as *const u8, compressedSize as usize);
    let dst = slice::from_raw_parts_mut(dest as *mut u8, dstCapacity as usize);
    let dict = if dictSize > 0 {
        slice::from_raw_parts(dictStart as *const u8, dictSize as usize)
    } else {
        &[]
    };
    decompress_block_partial_with_dict(src, dst, targetOutputSize as usize, dict)
        .map_or(-1, |n| n as c_int)
}

#[no_mangle]
pub extern "C" fn LZ4_decoderRingBufferSize(maxBlockSize: c_int) -> c_int {
    if maxBlockSize < 0 {
        return 0;
    }
    maxBlockSize.saturating_add((LZ4_DISTANCE_MAX + 1 + 14) as c_int)
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_decompress_fast(
    source: *const c_char,
    dest: *mut c_char,
    originalSize: c_int,
) -> c_int {
    if originalSize < 0 || source.is_null() || dest.is_null() {
        return -1;
    }
    let dst = slice::from_raw_parts_mut(dest as *mut u8, originalSize as usize);
    decompress_block_exact_ptr(source as *const u8, dst, &[]).map_or(-1, |n| n as c_int)
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_decompress_fast_usingDict(
    source: *const c_char,
    dest: *mut c_char,
    originalSize: c_int,
    dictStart: *const c_char,
    dictSize: c_int,
) -> c_int {
    if originalSize < 0
        || dictSize < 0
        || source.is_null()
        || dest.is_null()
        || (dictSize > 0 && dictStart.is_null())
    {
        return -1;
    }
    let dst = slice::from_raw_parts_mut(dest as *mut u8, originalSize as usize);
    let dict = if dictSize > 0 {
        slice::from_raw_parts(dictStart as *const u8, dictSize as usize)
    } else {
        &[]
    };
    decompress_block_exact_ptr(source as *const u8, dst, dict).map_or(-1, |n| n as c_int)
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_uncompress(
    source: *const c_char,
    dest: *mut c_char,
    outputSize: c_int,
) -> c_int {
    LZ4_decompress_fast(source, dest, outputSize)
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_uncompress_unknownOutputSize(
    source: *const c_char,
    dest: *mut c_char,
    isize: c_int,
    maxOutputSize: c_int,
) -> c_int {
    LZ4_decompress_safe(source, dest, isize, maxOutputSize)
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_decompress_safe_withPrefix64k(
    source: *const c_char,
    dest: *mut c_char,
    compressedSize: c_int,
    maxDstSize: c_int,
) -> c_int {
    LZ4_decompress_safe_usingDict(
        source,
        dest,
        compressedSize,
        maxDstSize,
        if maxDstSize > 0 {
            dest.sub(cmp::min(maxDstSize as usize, LZ4_DISTANCE_MAX))
        } else {
            dest
        },
        cmp::min(maxDstSize.max(0) as usize, LZ4_DISTANCE_MAX) as c_int,
    )
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_decompress_fast_withPrefix64k(
    source: *const c_char,
    dest: *mut c_char,
    originalSize: c_int,
) -> c_int {
    LZ4_decompress_fast_usingDict(
        source,
        dest,
        originalSize,
        if originalSize > 0 {
            dest.sub(cmp::min(originalSize as usize, LZ4_DISTANCE_MAX))
        } else {
            dest
        },
        cmp::min(originalSize.max(0) as usize, LZ4_DISTANCE_MAX) as c_int,
    )
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_decompress_fast_continue(
    stream: *mut LZ4StreamDecode,
    source: *const c_char,
    dest: *mut c_char,
    originalSize: c_int,
) -> c_int {
    if stream.is_null() || originalSize < 0 || source.is_null() || dest.is_null() {
        return -1;
    }
    let ctx = &mut *(stream as *mut DecodeStreamCtx);
    let dst = slice::from_raw_parts_mut(dest as *mut u8, originalSize as usize);
    match decompress_block_exact_ptr(source as *const u8, dst, &ctx.dictionary) {
        Some(n) => {
            append_hc_dictionary(&mut ctx.dictionary, dst);
            n as c_int
        }
        None => -1,
    }
}

#[no_mangle]
pub extern "C" fn LZ4F_isError(code: size_t) -> c_uint {
    (code >= usize::MAX - 1024) as c_uint
}

#[no_mangle]
pub extern "C" fn LZ4F_getErrorName(code: size_t) -> *const c_char {
    match code {
        ERROR_DST_TOO_SMALL => ERROR_DST_NAME.as_ptr() as *const c_char,
        ERROR_BAD_HEADER => ERROR_BAD_HEADER_NAME.as_ptr() as *const c_char,
        ERROR_CHECKSUM_INVALID => ERROR_CHECKSUM_NAME.as_ptr() as *const c_char,
        ERROR_GENERIC => ERROR_GENERIC_NAME.as_ptr() as *const c_char,
        _ => ERROR_OK_NAME.as_ptr() as *const c_char,
    }
}

#[no_mangle]
pub extern "C" fn LZ4F_getErrorCode(code: size_t) -> c_uint {
    match code {
        ERROR_DST_TOO_SMALL => 1,
        ERROR_BAD_HEADER => 2,
        ERROR_CHECKSUM_INVALID => 3,
        ERROR_GENERIC => 4,
        _ => 0,
    }
}

#[no_mangle]
pub extern "C" fn LZ4F_getVersion() -> c_uint {
    LZ4F_VERSION
}

#[no_mangle]
pub extern "C" fn LZ4F_compressionLevel_max() -> c_int {
    LZ4HC_CLEVEL_MAX
}

#[no_mangle]
pub extern "C" fn LZ4F_getBlockSize(blockSizeID: c_uint) -> size_t {
    if !(4..=7).contains(&blockSizeID) {
        return ERROR_BAD_HEADER;
    }
    block_max_size(blockSizeID as u8)
}

#[no_mangle]
pub unsafe extern "C" fn LZ4F_createCompressionContext(
    ctx: &mut LZ4FCompressionContext,
    version: c_uint,
) -> LZ4FErrorCode {
    if version != LZ4F_VERSION {
        return ERROR_GENERIC;
    }
    let inner = Box::new(CompressionCtx {
        prefs: FramePrefs::default(),
        content_hasher: XxHash32::new(0),
        dictionary: Vec::new(),
        external_dictionary: false,
        started: false,
    });
    ctx.0 = Box::into_raw(inner) as *mut c_void;
    0
}

#[no_mangle]
pub unsafe extern "C" fn LZ4F_freeCompressionContext(ctx: LZ4FCompressionContext) -> LZ4FErrorCode {
    if !ctx.0.is_null() {
        drop(Box::from_raw(ctx.0 as *mut CompressionCtx));
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn LZ4F_cctx_size(ctx: LZ4FCompressionContext) -> size_t {
    if ctx.0.is_null() {
        0
    } else {
        std::mem::size_of::<CompressionCtx>()
    }
}

#[no_mangle]
pub unsafe extern "C" fn LZ4F_compressBegin(
    ctx: LZ4FCompressionContext,
    dstBuffer: *mut u8,
    dstMaxSize: size_t,
    preferencesPtr: *const LZ4FPreferences,
) -> LZ4FErrorCode {
    if ctx.0.is_null() || dstBuffer.is_null() {
        return ERROR_GENERIC;
    }
    let inner = &mut *(ctx.0 as *mut CompressionCtx);
    inner.prefs = preferences_from_ptr(preferencesPtr);
    inner.content_hasher = XxHash32::new(0);
    inner.dictionary.clear();
    inner.external_dictionary = false;
    inner.started = true;
    let header = frame_header(inner.prefs);
    if dstMaxSize < header.len() {
        return ERROR_DST_TOO_SMALL;
    }
    ptr::copy_nonoverlapping(header.as_ptr(), dstBuffer, header.len());
    header.len()
}

#[no_mangle]
pub unsafe extern "C" fn LZ4F_compressBegin_usingDict(
    ctx: LZ4FCompressionContext,
    dstBuffer: *mut c_void,
    dstCapacity: size_t,
    dictBuffer: *const c_void,
    dictSize: size_t,
    preferencesPtr: *const LZ4FPreferences,
) -> LZ4FErrorCode {
    if dictSize > 0 && dictBuffer.is_null() {
        return ERROR_GENERIC;
    }
    let written = LZ4F_compressBegin(ctx, dstBuffer as *mut u8, dstCapacity, preferencesPtr);
    if LZ4F_isError(written) != 0 {
        return written;
    }
    if dictSize > 0 {
        let inner = &mut *(ctx.0 as *mut CompressionCtx);
        let dict = slice::from_raw_parts(dictBuffer as *const u8, dictSize);
        let keep = cmp::min(dict.len(), LZ4_DISTANCE_MAX);
        inner.dictionary.clear();
        inner
            .dictionary
            .extend_from_slice(&dict[dict.len() - keep..]);
        inner.external_dictionary = true;
    }
    written
}

#[no_mangle]
pub unsafe extern "C" fn LZ4F_createCDict(
    dictBuffer: *const c_void,
    dictSize: size_t,
) -> *mut LZ4FCDict {
    if dictSize > 0 && dictBuffer.is_null() {
        return ptr::null_mut();
    }
    let mut ctx = CDictCtx::default();
    if dictSize > 0 {
        let dict = slice::from_raw_parts(dictBuffer as *const u8, dictSize);
        let keep = cmp::min(dict.len(), LZ4_DISTANCE_MAX);
        ctx.dictionary.extend_from_slice(&dict[dict.len() - keep..]);
    }
    Box::into_raw(Box::new(ctx)) as *mut LZ4FCDict
}

#[no_mangle]
pub unsafe extern "C" fn LZ4F_freeCDict(cdict: *mut LZ4FCDict) {
    if !cdict.is_null() {
        drop(Box::from_raw(cdict as *mut CDictCtx));
    }
}

#[no_mangle]
pub unsafe extern "C" fn LZ4F_compressBegin_usingCDict(
    ctx: LZ4FCompressionContext,
    dstBuffer: *mut c_void,
    dstCapacity: size_t,
    cdict: *const LZ4FCDict,
    preferencesPtr: *const LZ4FPreferences,
) -> size_t {
    if cdict.is_null() {
        return LZ4F_compressBegin(ctx, dstBuffer as *mut u8, dstCapacity, preferencesPtr);
    }
    let dict = &*(cdict as *const CDictCtx);
    LZ4F_compressBegin_usingDict(
        ctx,
        dstBuffer,
        dstCapacity,
        dict.dictionary.as_ptr() as *const c_void,
        dict.dictionary.len(),
        preferencesPtr,
    )
}

#[no_mangle]
pub unsafe extern "C" fn LZ4F_compressBound(
    srcSize: size_t,
    preferencesPtr: *const LZ4FPreferences,
) -> size_t {
    let prefs = preferences_from_ptr(preferencesPtr);
    let checksums = if prefs.block_checksum { 4 } else { 0 };
    let block_max = block_max_size(prefs.block_size_id);
    let blocks = if srcSize == 0 {
        1
    } else {
        srcSize.div_ceil(block_max)
    };
    srcSize + blocks * (4 + checksums) + 16
}

#[no_mangle]
pub unsafe extern "C" fn LZ4F_compressFrameBound(
    srcSize: size_t,
    preferencesPtr: *const LZ4FPreferences,
) -> size_t {
    LZ4F_compressBound(srcSize, preferencesPtr) + 19 + 4
}

#[no_mangle]
pub unsafe extern "C" fn LZ4F_compressFrame(
    dstBuffer: *mut c_void,
    dstCapacity: size_t,
    srcBuffer: *const c_void,
    srcSize: size_t,
    preferencesPtr: *const LZ4FPreferences,
) -> size_t {
    if dstBuffer.is_null() || (srcSize > 0 && srcBuffer.is_null()) {
        return ERROR_GENERIC;
    }
    let mut ctx = LZ4FCompressionContext(ptr::null_mut());
    let code = LZ4F_createCompressionContext(&mut ctx, LZ4F_VERSION);
    if LZ4F_isError(code) != 0 {
        return code;
    }

    let dst = dstBuffer as *mut u8;
    let mut pos = LZ4F_compressBegin(ctx, dst, dstCapacity, preferencesPtr);
    if LZ4F_isError(pos) != 0 {
        LZ4F_freeCompressionContext(ctx);
        return pos;
    }
    let update = LZ4F_compressUpdate(
        ctx,
        dst.add(pos),
        dstCapacity.saturating_sub(pos),
        srcBuffer as *const u8,
        srcSize,
        ptr::null(),
    );
    if LZ4F_isError(update) != 0 {
        LZ4F_freeCompressionContext(ctx);
        return update;
    }
    pos += update;
    let end = LZ4F_compressEnd(
        ctx,
        dst.add(pos),
        dstCapacity.saturating_sub(pos),
        ptr::null(),
    );
    LZ4F_freeCompressionContext(ctx);
    if LZ4F_isError(end) != 0 {
        return end;
    }
    pos + end
}

#[no_mangle]
pub unsafe extern "C" fn LZ4F_compressFrame_usingCDict(
    ctx: LZ4FCompressionContext,
    dstBuffer: *mut c_void,
    dstCapacity: size_t,
    srcBuffer: *const c_void,
    srcSize: size_t,
    cdict: *const LZ4FCDict,
    preferencesPtr: *const LZ4FPreferences,
) -> size_t {
    if ctx.0.is_null() || dstBuffer.is_null() || (srcSize > 0 && srcBuffer.is_null()) {
        return ERROR_GENERIC;
    }
    let dst = dstBuffer as *mut u8;
    let mut pos = LZ4F_compressBegin_usingCDict(ctx, dstBuffer, dstCapacity, cdict, preferencesPtr);
    if LZ4F_isError(pos) != 0 {
        return pos;
    }
    let update = LZ4F_compressUpdate(
        ctx,
        dst.add(pos),
        dstCapacity.saturating_sub(pos),
        srcBuffer as *const u8,
        srcSize,
        ptr::null(),
    );
    if LZ4F_isError(update) != 0 {
        return update;
    }
    pos += update;
    let end = LZ4F_compressEnd(
        ctx,
        dst.add(pos),
        dstCapacity.saturating_sub(pos),
        ptr::null(),
    );
    if LZ4F_isError(end) != 0 {
        return end;
    }
    pos + end
}

#[no_mangle]
pub unsafe extern "C" fn LZ4F_compressUpdate(
    ctx: LZ4FCompressionContext,
    dstBuffer: *mut u8,
    dstCapacity: size_t,
    srcBuffer: *const u8,
    srcSize: size_t,
    _cOptPtr: *const LZ4FCompressOptions,
) -> size_t {
    if ctx.0.is_null() || dstBuffer.is_null() || (srcSize > 0 && srcBuffer.is_null()) {
        return ERROR_GENERIC;
    }
    let inner = &mut *(ctx.0 as *mut CompressionCtx);
    let src = if srcSize > 0 {
        slice::from_raw_parts(srcBuffer, srcSize)
    } else {
        &[]
    };
    let block_max = block_max_size(inner.prefs.block_size_id);
    if src.len() > block_max {
        let dst = slice::from_raw_parts_mut(dstBuffer, dstCapacity);
        let mut written = 0usize;
        for chunk in src.chunks(block_max) {
            let n = compress_frame_update_block(inner, chunk, &mut dst[written..]);
            if LZ4F_isError(n) != 0 {
                return n;
            }
            written += n;
        }
        return written;
    }

    let dst = slice::from_raw_parts_mut(dstBuffer, dstCapacity);
    compress_frame_update_block(inner, src, dst)
}

fn compress_frame_update_block(
    inner: &mut CompressionCtx,
    src: &[u8],
    dst: &mut [u8],
) -> size_t {
    let checksum_len = if inner.prefs.block_checksum { 4 } else { 0 };
    let raw_needed = 4 + src.len() + checksum_len;
    if dst.len() < raw_needed {
        return ERROR_DST_TOO_SMALL;
    }

    let compressed_len = compress_frame_block(src, &mut dst[4..], &inner.prefs, &inner.dictionary);
    let (block_len, raw) = match compressed_len {
        Some(len) if len < src.len() => (len, false),
        _ => {
            dst[4..4 + src.len()].copy_from_slice(src);
            (src.len(), true)
        }
    };

    let block_size = (block_len as u32) | if raw { 0x8000_0000 } else { 0 };
    dst[..4].copy_from_slice(&block_size.to_le_bytes());
    inner.content_hasher.update(src);
    if !inner.prefs.block_independent {
        append_hc_dictionary(&mut inner.dictionary, src);
        inner.external_dictionary = false;
    } else if inner.external_dictionary {
        inner.dictionary.clear();
        inner.external_dictionary = false;
    }
    let needed = 4 + block_len + checksum_len;
    if inner.prefs.block_checksum {
        let checksum = xxhash32(&dst[4..4 + block_len], 0);
        dst[4 + block_len..needed].copy_from_slice(&checksum.to_le_bytes());
    }
    needed
}

#[no_mangle]
pub unsafe extern "C" fn LZ4F_uncompressedUpdate(
    ctx: LZ4FCompressionContext,
    dstBuffer: *mut c_void,
    dstCapacity: size_t,
    srcBuffer: *const c_void,
    srcSize: size_t,
    _cOptPtr: *const LZ4FCompressOptions,
) -> size_t {
    if ctx.0.is_null() || dstBuffer.is_null() || (srcSize > 0 && srcBuffer.is_null()) {
        return ERROR_GENERIC;
    }
    let inner = &mut *(ctx.0 as *mut CompressionCtx);
    let checksum_len = if inner.prefs.block_checksum { 4 } else { 0 };
    let needed = 4 + srcSize + checksum_len;
    if dstCapacity < needed {
        return ERROR_DST_TOO_SMALL;
    }
    let src = if srcSize > 0 {
        slice::from_raw_parts(srcBuffer as *const u8, srcSize)
    } else {
        &[]
    };
    let dst = slice::from_raw_parts_mut(dstBuffer as *mut u8, dstCapacity);
    let block_size = (srcSize as u32) | 0x8000_0000;
    dst[..4].copy_from_slice(&block_size.to_le_bytes());
    dst[4..4 + srcSize].copy_from_slice(src);
    if inner.prefs.block_checksum {
        let checksum = xxhash32(&dst[4..4 + srcSize], 0);
        dst[4 + srcSize..needed].copy_from_slice(&checksum.to_le_bytes());
    }
    inner.content_hasher.update(src);
    if !inner.prefs.block_independent {
        append_hc_dictionary(&mut inner.dictionary, src);
    }
    needed
}

#[no_mangle]
pub unsafe extern "C" fn LZ4F_flush(
    _ctx: LZ4FCompressionContext,
    _dstBuffer: *mut u8,
    _dstCapacity: size_t,
    _cOptPtr: *const LZ4FCompressOptions,
) -> size_t {
    0
}

#[no_mangle]
pub unsafe extern "C" fn LZ4F_compressEnd(
    ctx: LZ4FCompressionContext,
    dstBuffer: *mut u8,
    dstCapacity: size_t,
    _cOptPtr: *const LZ4FCompressOptions,
) -> size_t {
    if ctx.0.is_null() || dstBuffer.is_null() {
        return ERROR_GENERIC;
    }
    let inner = &mut *(ctx.0 as *mut CompressionCtx);
    let needed = 4 + if inner.prefs.content_checksum { 4 } else { 0 };
    if dstCapacity < needed {
        return ERROR_DST_TOO_SMALL;
    }
    let dst = slice::from_raw_parts_mut(dstBuffer, dstCapacity);
    dst[..4].copy_from_slice(&0u32.to_le_bytes());
    if inner.prefs.content_checksum {
        dst[4..8].copy_from_slice(&inner.content_hasher.digest().to_le_bytes());
    }
    needed
}

#[no_mangle]
pub unsafe extern "C" fn LZ4F_createDecompressionContext(
    ctx: &mut LZ4FDecompressionContext,
    version: c_uint,
) -> LZ4FErrorCode {
    if version != LZ4F_VERSION {
        return ERROR_GENERIC;
    }
    ctx.0 = Box::into_raw(Box::<DecompressionCtx>::default()) as *mut c_void;
    0
}

#[no_mangle]
pub unsafe extern "C" fn LZ4F_freeDecompressionContext(
    ctx: LZ4FDecompressionContext,
) -> LZ4FErrorCode {
    if !ctx.0.is_null() {
        drop(Box::from_raw(ctx.0 as *mut DecompressionCtx));
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn LZ4F_dctx_size(ctx: LZ4FDecompressionContext) -> size_t {
    if ctx.0.is_null() {
        0
    } else {
        std::mem::size_of::<DecompressionCtx>()
    }
}

#[no_mangle]
pub unsafe extern "C" fn LZ4F_getFrameInfo(
    ctx: LZ4FDecompressionContext,
    frameInfoPtr: *mut LZ4FFrameInfo,
    srcBuffer: *const u8,
    srcSizePtr: *mut size_t,
) -> size_t {
    if ctx.0.is_null() || frameInfoPtr.is_null() || srcSizePtr.is_null() {
        return ERROR_GENERIC;
    }
    let src_size = *srcSizePtr;
    if src_size < 7 || srcBuffer.is_null() {
        return ERROR_BAD_HEADER;
    }
    let src = slice::from_raw_parts(srcBuffer, src_size);
    if is_skippable_magic_prefix(src) {
        let Some(skip_len) = parse_skippable_frame_len(src) else {
            return ERROR_BAD_HEADER;
        };
        *srcSizePtr = cmp::min(skip_len, src_size);
        *frameInfoPtr = LZ4FFrameInfo {
            block_size_id: BlockSize::Default,
            block_mode: BlockMode::Independent,
            content_checksum_flag: ContentChecksum::NoChecksum,
            frame_type: FrameType::SkippableFrame,
            content_size: 0,
            dict_id: 0,
            block_checksum_flag: BlockChecksum::NoBlockChecksum,
        };
        return 0;
    }
    let Some((prefs, header_len)) = parse_frame_header(src) else {
        return ERROR_BAD_HEADER;
    };
    *srcSizePtr = header_len;
    *frameInfoPtr = LZ4FFrameInfo {
        block_size_id: block_size_enum(prefs.block_size_id),
        block_mode: if prefs.block_independent {
            BlockMode::Independent
        } else {
            BlockMode::Linked
        },
        content_checksum_flag: if prefs.content_checksum {
            ContentChecksum::ChecksumEnabled
        } else {
            ContentChecksum::NoChecksum
        },
        frame_type: FrameType::Frame,
        content_size: prefs.content_size,
        dict_id: 0,
        block_checksum_flag: if prefs.block_checksum {
            BlockChecksum::BlockChecksumEnabled
        } else {
            BlockChecksum::NoBlockChecksum
        },
    };
    0
}

#[no_mangle]
pub unsafe extern "C" fn LZ4F_headerSize(src: *const c_void, srcSize: size_t) -> size_t {
    if src.is_null() || srcSize < 6 {
        return ERROR_BAD_HEADER;
    }
    let src = slice::from_raw_parts(src as *const u8, srcSize);
    if is_skippable_magic_prefix(src) {
        return 8;
    }
    expected_frame_header_len(src).unwrap_or(ERROR_BAD_HEADER)
}

#[no_mangle]
pub unsafe extern "C" fn LZ4F_decompress(
    ctx: LZ4FDecompressionContext,
    dstBuffer: *mut u8,
    dstSizePtr: *mut size_t,
    srcBuffer: *const u8,
    srcSizePtr: *mut size_t,
    _dOptPtr: *const LZ4FDecompressOptions,
) -> size_t {
    if ctx.0.is_null() || dstSizePtr.is_null() || srcSizePtr.is_null() {
        return ERROR_GENERIC;
    }
    let inner = &mut *(ctx.0 as *mut DecompressionCtx);
    let src_size = *srcSizePtr;
    if src_size > 0 {
        if srcBuffer.is_null() {
            return ERROR_GENERIC;
        }
        inner
            .input
            .extend_from_slice(slice::from_raw_parts(srcBuffer, src_size));
    }

    let dst_capacity = *dstSizePtr;
    if dst_capacity > 0 && dstBuffer.is_null() {
        return ERROR_GENERIC;
    }

    if pending_is_empty(inner) && dst_capacity > 0 {
        if let Err(code) = parse_frame_header_if_available(inner) {
            return code;
        }
        if let Some(result) = try_decompress_frame_block_to_dst(
            inner,
            slice::from_raw_parts_mut(dstBuffer, dst_capacity),
        ) {
            match result {
                Ok(written) => {
                    *srcSizePtr = consumed_from_call(inner.done, src_size, inner.input.len());
                    *dstSizePtr = written;
                    return if inner.done && pending_is_empty(inner) {
                        0
                    } else {
                        frame_hint(inner)
                    };
                }
                Err(code) => return code,
            }
        }
    }

    if let Err(code) = parse_available_frame(inner) {
        return code;
    }
    *srcSizePtr = consumed_from_call(inner.done, src_size, inner.input.len());
    let pending_len = pending_len(inner);
    let to_copy = cmp::min(dst_capacity, pending_len);
    if to_copy > 0 {
        ptr::copy_nonoverlapping(
            inner.pending.as_ptr().add(inner.pending_pos),
            dstBuffer,
            to_copy,
        );
        inner.pending_pos += to_copy;
        if inner.pending_pos == inner.pending.len() {
            inner.pending.clear();
            inner.pending_pos = 0;
        }
    }
    *dstSizePtr = to_copy;
    if pending_is_empty(inner) && !inner.done {
        if let Err(code) = parse_available_frame(inner) {
            return code;
        }
    }
    if inner.done && pending_is_empty(inner) {
        0
    } else {
        frame_hint(inner)
    }
}

#[no_mangle]
pub unsafe extern "C" fn LZ4F_decompress_usingDict(
    ctx: LZ4FDecompressionContext,
    dstBuffer: *mut c_void,
    dstSizePtr: *mut size_t,
    srcBuffer: *const c_void,
    srcSizePtr: *mut size_t,
    dict: *const c_void,
    dictSize: size_t,
    decompressOptionsPtr: *const LZ4FDecompressOptions,
) -> size_t {
    if ctx.0.is_null() {
        return ERROR_GENERIC;
    }
    if dictSize > 0 && dict.is_null() {
        return ERROR_GENERIC;
    }
    let inner = &mut *(ctx.0 as *mut DecompressionCtx);
    if !inner.parsed_header && dictSize > 0 {
        let dict = slice::from_raw_parts(dict as *const u8, dictSize);
        let keep = cmp::min(dict.len(), LZ4_DISTANCE_MAX);
        inner.dictionary.clear();
        inner
            .dictionary
            .extend_from_slice(&dict[dict.len() - keep..]);
        inner.external_dictionary = true;
    }
    LZ4F_decompress(
        ctx,
        dstBuffer as *mut u8,
        dstSizePtr,
        srcBuffer as *const u8,
        srcSizePtr,
        decompressOptionsPtr,
    )
}

#[no_mangle]
pub unsafe extern "C" fn LZ4F_resetDecompressionContext(ctx: LZ4FDecompressionContext) {
    if !ctx.0.is_null() {
        *(ctx.0 as *mut DecompressionCtx) = DecompressionCtx::default();
    }
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_createStream() -> *mut LZ4StreamEncode {
    Box::into_raw(Box::<EncodeStreamCtx>::default()) as *mut LZ4StreamEncode
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_freeStream(stream: *mut LZ4StreamEncode) -> c_int {
    if !stream.is_null() {
        drop(Box::from_raw(stream as *mut EncodeStreamCtx));
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_compress_continue(
    stream: *mut LZ4StreamEncode,
    src: *const c_char,
    dst: *mut c_char,
    srcSize: c_int,
    dstCapacity: c_int,
) -> c_int {
    if stream.is_null() || srcSize < 0 || dstCapacity <= 0 || src.is_null() || dst.is_null() {
        return 0;
    }
    let ctx = &mut *(stream as *mut EncodeStreamCtx);
    let src_slice = slice::from_raw_parts(src as *const u8, srcSize as usize);
    let dst_slice = slice::from_raw_parts_mut(dst as *mut u8, dstCapacity as usize);
    let written = if ctx.dictionary.is_empty() {
        compress_block(src_slice, dst_slice)
    } else {
        compress_block_with_dict(src_slice, dst_slice, &ctx.dictionary)
    }
    .map_or(0, |n| n as c_int);
    if written > 0 {
        append_hc_dictionary(&mut ctx.dictionary, src_slice);
    }
    written
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_compress_fast_continue(
    stream: *mut LZ4StreamEncode,
    src: *const c_char,
    dst: *mut c_char,
    srcSize: c_int,
    dstCapacity: c_int,
    _acceleration: c_int,
) -> c_int {
    LZ4_compress_continue(stream, src, dst, srcSize, dstCapacity)
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_compress_limitedOutput_continue(
    stream: *mut LZ4StreamEncode,
    src: *const c_char,
    dst: *mut c_char,
    srcSize: c_int,
    maxOutputSize: c_int,
) -> c_int {
    LZ4_compress_continue(stream, src, dst, srcSize, maxOutputSize)
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_resetStream_fast(stream: *mut LZ4StreamEncode) {
    if !stream.is_null() {
        (*(stream as *mut EncodeStreamCtx)).dictionary.clear();
    }
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_resetStream(stream: *mut LZ4StreamEncode) {
    LZ4_resetStream_fast(stream)
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_initStream(
    stateBuffer: *mut c_void,
    _size: size_t,
) -> *mut LZ4StreamEncode {
    if stateBuffer.is_null() {
        return ptr::null_mut();
    }
    ptr::write(
        stateBuffer as *mut EncodeStreamCtx,
        EncodeStreamCtx::default(),
    );
    stateBuffer as *mut LZ4StreamEncode
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_create(_inputBuffer: *mut c_char) -> *mut c_void {
    LZ4_createStream() as *mut c_void
}

#[no_mangle]
pub extern "C" fn LZ4_sizeofStreamState() -> c_int {
    LZ4_sizeofState()
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_resetStreamState(
    state: *mut c_void,
    _inputBuffer: *mut c_char,
) -> c_int {
    if state.is_null() {
        return 1;
    }
    ptr::write(state as *mut EncodeStreamCtx, EncodeStreamCtx::default());
    0
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_slideInputBuffer(_state: *mut c_void) -> *mut c_char {
    ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_attach_dictionary(
    workingStream: *mut LZ4StreamEncode,
    dictionaryStream: *const LZ4StreamEncode,
) {
    if workingStream.is_null() {
        return;
    }
    let working = &mut *(workingStream as *mut EncodeStreamCtx);
    working.dictionary.clear();
    if !dictionaryStream.is_null() {
        let dictionary = &*(dictionaryStream as *const EncodeStreamCtx);
        working.dictionary.extend_from_slice(&dictionary.dictionary);
    }
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_loadDict(
    stream: *mut LZ4StreamEncode,
    dictionary: *const c_char,
    dictSize: c_int,
) -> c_int {
    if stream.is_null() || dictSize < 0 || (dictSize > 0 && dictionary.is_null()) {
        return 0;
    }
    let ctx = &mut *(stream as *mut EncodeStreamCtx);
    ctx.dictionary.clear();
    if dictSize > 0 {
        let dict = slice::from_raw_parts(dictionary as *const u8, dictSize as usize);
        let keep = cmp::min(dict.len(), LZ4_DISTANCE_MAX);
        ctx.dictionary.extend_from_slice(&dict[dict.len() - keep..]);
    }
    ctx.dictionary.len() as c_int
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_loadDictSlow(
    stream: *mut LZ4StreamEncode,
    dictionary: *const c_char,
    dictSize: c_int,
) -> c_int {
    LZ4_loadDict(stream, dictionary, dictSize)
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_saveDict(
    stream: *mut LZ4StreamEncode,
    safeBuffer: *mut c_char,
    maxDictSize: c_int,
) -> c_int {
    if stream.is_null() || maxDictSize < 0 || (maxDictSize > 0 && safeBuffer.is_null()) {
        return 0;
    }
    let ctx = &mut *(stream as *mut EncodeStreamCtx);
    let keep = cmp::min(
        ctx.dictionary.len(),
        cmp::min(maxDictSize as usize, LZ4_DISTANCE_MAX),
    );
    if keep > 0 {
        ptr::copy_nonoverlapping(
            ctx.dictionary[ctx.dictionary.len() - keep..].as_ptr(),
            safeBuffer as *mut u8,
            keep,
        );
        ctx.dictionary = ctx.dictionary[ctx.dictionary.len() - keep..].to_vec();
    } else {
        ctx.dictionary.clear();
    }
    keep as c_int
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_createStreamHC() -> *mut LZ4StreamHC {
    Box::into_raw(Box::<HcStreamCtx>::default()) as *mut LZ4StreamHC
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_freeStreamHC(stream: *mut LZ4StreamHC) -> c_int {
    if !stream.is_null() {
        drop(Box::from_raw(stream as *mut HcStreamCtx));
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_resetStreamHC_fast(stream: *mut LZ4StreamHC, compressionLevel: c_int) {
    if !stream.is_null() {
        let ctx = &mut *(stream as *mut HcStreamCtx);
        ctx.compression_level = normalize_hc_level(compressionLevel);
        ctx.dictionary.clear();
    }
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_resetStreamHC(stream: *mut LZ4StreamHC, compressionLevel: c_int) {
    LZ4_resetStreamHC_fast(stream, compressionLevel)
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_initStreamHC(buffer: *mut c_void, _size: size_t) -> *mut LZ4StreamHC {
    if buffer.is_null() {
        return ptr::null_mut();
    }
    ptr::write(buffer as *mut HcStreamCtx, HcStreamCtx::default());
    buffer as *mut LZ4StreamHC
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_setCompressionLevel(
    stream: *mut LZ4StreamHC,
    compressionLevel: c_int,
) {
    if !stream.is_null() {
        (*(stream as *mut HcStreamCtx)).compression_level = normalize_hc_level(compressionLevel);
    }
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_favorDecompressionSpeed(_stream: *mut LZ4StreamHC, _favor: c_int) {}

#[no_mangle]
pub unsafe extern "C" fn LZ4_attach_HC_dictionary(
    working_stream: *mut LZ4StreamHC,
    dictionary_stream: *const LZ4StreamHC,
) {
    if working_stream.is_null() {
        return;
    }
    let working = &mut *(working_stream as *mut HcStreamCtx);
    working.dictionary.clear();
    if !dictionary_stream.is_null() {
        let dictionary = &*(dictionary_stream as *const HcStreamCtx);
        working.dictionary.extend_from_slice(&dictionary.dictionary);
    }
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_loadDictHC(
    stream: *mut LZ4StreamHC,
    dictionary: *const c_char,
    dictSize: c_int,
) -> c_int {
    if stream.is_null() || dictSize < 0 || (dictSize > 0 && dictionary.is_null()) {
        return 0;
    }
    let ctx = &mut *(stream as *mut HcStreamCtx);
    let keep = cmp::min(dictSize as usize, LZ4_DISTANCE_MAX);
    ctx.dictionary.clear();
    if keep > 0 {
        let dict = slice::from_raw_parts(dictionary as *const u8, dictSize as usize);
        ctx.dictionary.extend_from_slice(&dict[dict.len() - keep..]);
    }
    keep as c_int
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_saveDictHC(
    stream: *mut LZ4StreamHC,
    safeBuffer: *mut c_char,
    maxDictSize: c_int,
) -> c_int {
    if stream.is_null() || maxDictSize < 0 || (maxDictSize > 0 && safeBuffer.is_null()) {
        return 0;
    }
    let ctx = &mut *(stream as *mut HcStreamCtx);
    let keep = cmp::min(
        ctx.dictionary.len(),
        cmp::min(maxDictSize as usize, LZ4_DISTANCE_MAX),
    );
    if keep > 0 {
        ptr::copy_nonoverlapping(
            ctx.dictionary[ctx.dictionary.len() - keep..].as_ptr(),
            safeBuffer as *mut u8,
            keep,
        );
        ctx.dictionary = ctx.dictionary[ctx.dictionary.len() - keep..].to_vec();
    } else {
        ctx.dictionary.clear();
    }
    keep as c_int
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_compress_HC_continue(
    stream: *mut LZ4StreamHC,
    src: *const c_char,
    dst: *mut c_char,
    srcSize: c_int,
    maxDstSize: c_int,
) -> c_int {
    if stream.is_null() {
        return 0;
    }
    let ctx = &mut *(stream as *mut HcStreamCtx);
    if srcSize < 0 || maxDstSize <= 0 || src.is_null() || dst.is_null() {
        return 0;
    }
    let src_slice = slice::from_raw_parts(src as *const u8, srcSize as usize);
    let dst_slice = slice::from_raw_parts_mut(dst as *mut u8, maxDstSize as usize);
    let written = if ctx.dictionary.is_empty() {
        compress_block_hc(src_slice, dst_slice, ctx.compression_level)
    } else {
        compress_block_hc_with_dict(src_slice, dst_slice, &ctx.dictionary, ctx.compression_level)
    }
    .map_or(0, |n| n as c_int);
    if written > 0 && srcSize > 0 && !src.is_null() {
        append_hc_dictionary(&mut ctx.dictionary, src_slice);
    }
    written
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_compress_HC_continue_destSize(
    stream: *mut LZ4StreamHC,
    src: *const c_char,
    dst: *mut c_char,
    srcSizePtr: *mut c_int,
    targetDstSize: c_int,
) -> c_int {
    if stream.is_null() {
        return 0;
    }
    let ctx = &mut *(stream as *mut HcStreamCtx);
    let original = if !srcSizePtr.is_null() {
        *srcSizePtr
    } else {
        0
    };
    if src.is_null()
        || dst.is_null()
        || srcSizePtr.is_null()
        || *srcSizePtr < 0
        || targetDstSize <= 0
    {
        return 0;
    }
    let src_slice = slice::from_raw_parts(src as *const u8, *srcSizePtr as usize);
    let dst_slice = slice::from_raw_parts_mut(dst as *mut u8, targetDstSize as usize);
    let result = if ctx.dictionary.is_empty() {
        compress_hc_dest_size(src_slice, dst_slice, ctx.compression_level)
    } else {
        compress_hc_dest_size_with_dict(
            src_slice,
            dst_slice,
            &ctx.dictionary,
            ctx.compression_level,
        )
    };
    let Some((consumed, written_usize)) = result else {
        return 0;
    };
    *srcSizePtr = consumed as c_int;
    let written = written_usize as c_int;
    if written > 0 && !src.is_null() && !srcSizePtr.is_null() && *srcSizePtr > 0 {
        let consumed = cmp::min(*srcSizePtr, original) as usize;
        let src_slice = slice::from_raw_parts(src as *const u8, consumed);
        append_hc_dictionary(&mut ctx.dictionary, src_slice);
    }
    written
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_compressHC_continue(
    stream: *mut LZ4StreamHC,
    src: *const c_char,
    dst: *mut c_char,
    srcSize: c_int,
) -> c_int {
    LZ4_compress_HC_continue(stream, src, dst, srcSize, LZ4_compressBound(srcSize))
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_compressHC_limitedOutput_continue(
    stream: *mut LZ4StreamHC,
    src: *const c_char,
    dst: *mut c_char,
    srcSize: c_int,
    maxDstSize: c_int,
) -> c_int {
    LZ4_compress_HC_continue(stream, src, dst, srcSize, maxDstSize)
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_compressHC2_continue(
    stream: *mut c_void,
    src: *const c_char,
    dst: *mut c_char,
    srcSize: c_int,
    cLevel: c_int,
) -> c_int {
    if stream.is_null() {
        return 0;
    }
    LZ4_compress_HC(src, dst, srcSize, LZ4_compressBound(srcSize), cLevel)
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_compressHC2_limitedOutput_continue(
    stream: *mut c_void,
    src: *const c_char,
    dst: *mut c_char,
    srcSize: c_int,
    maxDstSize: c_int,
    cLevel: c_int,
) -> c_int {
    if stream.is_null() {
        return 0;
    }
    LZ4_compress_HC(src, dst, srcSize, maxDstSize, cLevel)
}

#[no_mangle]
pub extern "C" fn LZ4_sizeofStreamStateHC() -> c_int {
    LZ4_sizeofStateHC()
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_createHC(_inputBuffer: *const c_char) -> *mut c_void {
    LZ4_createStreamHC() as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_freeHC(stream: *mut c_void) -> c_int {
    LZ4_freeStreamHC(stream as *mut LZ4StreamHC)
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_slideInputBufferHC(_stream: *mut c_void) -> *mut c_char {
    ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_resetStreamStateHC(
    state: *mut c_void,
    _inputBuffer: *mut c_char,
) -> c_int {
    if state.is_null() {
        return 1;
    }
    ptr::write(state as *mut HcStreamCtx, HcStreamCtx::default());
    0
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_createStreamDecode() -> *mut LZ4StreamDecode {
    Box::into_raw(Box::<DecodeStreamCtx>::default()) as *mut LZ4StreamDecode
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_freeStreamDecode(stream: *mut LZ4StreamDecode) -> c_int {
    if !stream.is_null() {
        drop(Box::from_raw(stream as *mut DecodeStreamCtx));
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_setStreamDecode(
    stream: *mut LZ4StreamDecode,
    dictionary: *const c_char,
    dictSize: c_int,
) -> c_int {
    if stream.is_null() || dictSize < 0 || (dictSize > 0 && dictionary.is_null()) {
        return 0;
    }
    let ctx = &mut *(stream as *mut DecodeStreamCtx);
    ctx.dictionary.clear();
    if dictSize > 0 {
        let dict = slice::from_raw_parts(dictionary as *const u8, dictSize as usize);
        let keep = cmp::min(dict.len(), LZ4_DISTANCE_MAX);
        ctx.dictionary.extend_from_slice(&dict[dict.len() - keep..]);
    }
    1
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_decompress_safe_continue(
    stream: *mut LZ4StreamDecode,
    src: *const c_char,
    dst: *mut c_char,
    srcSize: c_int,
    dstCapacity: c_int,
) -> c_int {
    if stream.is_null() || srcSize < 0 || dstCapacity < 0 || src.is_null() || dst.is_null() {
        return -1;
    }
    let ctx = &mut *(stream as *mut DecodeStreamCtx);
    let src = slice::from_raw_parts(src as *const u8, srcSize as usize);
    let dst = slice::from_raw_parts_mut(dst as *mut u8, dstCapacity as usize);
    match decompress_block_with_dict(src, dst, &ctx.dictionary) {
        Some(n) => {
            append_hc_dictionary(&mut ctx.dictionary, &dst[..n]);
            n as c_int
        }
        None => -1,
    }
}

fn compress_block(src: &[u8], dst: &mut [u8]) -> Option<usize> {
    if src.is_empty() {
        return emit_last_literals(src, dst, 0, 0);
    }
    if src.len() < MFLIMIT + 1 {
        return emit_last_literals(src, dst, 0, 0);
    }
    let mut table = vec![usize::MAX; HASH_SIZE];
    let mut ip = 0usize;
    let mut anchor = 0usize;
    let mut op = 0usize;
    let mflimit_plus_one = src.len() - MFLIMIT + 1;
    let match_limit = src.len() - LAST_LITERALS;

    table[hash4(src, ip)] = ip;
    ip += 1;
    let mut forward_h = hash4(src, ip);

    loop {
        let mut forward_ip = ip;
        let mut step = 1usize;
        let mut search_match_nb = 1usize << 6;
        let mut ref_pos;

        loop {
            let h = forward_h;
            ip = forward_ip;
            forward_ip += step;
            step = search_match_nb >> 6;
            search_match_nb += 1;

            if forward_ip > mflimit_plus_one {
                return emit_last_literals(src, dst, anchor, op);
            }

            ref_pos = table[h];
            forward_h = hash4(src, forward_ip);
            table[h] = ip;

            if ref_pos != usize::MAX
                && ip > ref_pos
                && ip - ref_pos <= LZ4_DISTANCE_MAX
                && src[ref_pos..ref_pos + MINMATCH] == src[ip..ip + MINMATCH]
            {
                break;
            }
        }

        while ip > anchor && ref_pos > 0 && src[ip - 1] == src[ref_pos - 1] {
            ip -= 1;
            ref_pos -= 1;
        }

        loop {
            let match_len =
                MINMATCH + count_match(src, ip + MINMATCH, ref_pos + MINMATCH, match_limit);
            op = encode_sequence(src, dst, anchor, ip, match_len, ip - ref_pos, op)?;
            ip += match_len;
            anchor = ip;

            if ip >= mflimit_plus_one {
                return emit_last_literals(src, dst, anchor, op);
            }

            table[hash4(src, ip - 2)] = ip - 2;
            let h = hash4(src, ip);
            ref_pos = table[h];
            table[h] = ip;
            if ref_pos != usize::MAX
                && ip > ref_pos
                && ip - ref_pos <= LZ4_DISTANCE_MAX
                && src[ref_pos..ref_pos + MINMATCH] == src[ip..ip + MINMATCH]
            {
                continue;
            }

            ip += 1;
            forward_h = hash4(src, ip);
            break;
        }
    }
}

fn compress_dest_size(src: &[u8], dst: &mut [u8]) -> Option<(usize, usize)> {
    if src.is_empty() {
        let written = compress_block(src, dst)?;
        return Some((0, written));
    }

    let mut low = 0usize;
    let mut high = src.len();
    let mut best: Option<(usize, usize, Vec<u8>)> = None;
    while low <= high {
        let mid = low + (high - low) / 2;
        let mut candidate = vec![0u8; dst.len()];
        match compress_block(&src[..mid], &mut candidate) {
            Some(written) if written <= dst.len() => {
                best = Some((mid, written, candidate));
                low = mid + 1;
            }
            _ => {
                if mid == 0 {
                    break;
                }
                high = mid - 1;
            }
        }
    }

    let (consumed, written, candidate) = best?;
    dst[..written].copy_from_slice(&candidate[..written]);
    Some((consumed, written))
}

fn compress_block_with_dict(src: &[u8], dst: &mut [u8], dict: &[u8]) -> Option<usize> {
    if src.is_empty() {
        return emit_last_literals(src, dst, 0, 0);
    }
    if src.len() < MFLIMIT + 1 {
        return emit_last_literals(src, dst, 0, 0);
    }
    let dict_keep = cmp::min(dict.len(), LZ4_DISTANCE_MAX);
    let dict = &dict[dict.len() - dict_keep..];
    if dict.is_empty() {
        return compress_block(src, dst);
    }

    let mut full = Vec::with_capacity(dict.len() + src.len());
    full.extend_from_slice(dict);
    full.extend_from_slice(src);
    let base = dict.len();
    let mut table = vec![usize::MAX; HASH_SIZE];
    let seed_end = full.len().saturating_sub(MINMATCH - 1);
    for pos in 0..cmp::min(base, seed_end) {
        table[hash4(&full, pos)] = pos;
    }

    let mut ip = base;
    let mut anchor = base;
    let mut op = 0usize;
    let mflimit_plus_one = base + src.len() - MFLIMIT + 1;
    let match_limit = base + src.len() - LAST_LITERALS;

    table[hash4(&full, ip)] = ip;
    ip += 1;
    let mut forward_h = hash4(&full, ip);

    loop {
        let mut forward_ip = ip;
        let mut step = 1usize;
        let mut search_match_nb = 1usize << 6;
        let mut ref_pos;

        loop {
            let h = forward_h;
            ip = forward_ip;
            forward_ip += step;
            step = search_match_nb >> 6;
            search_match_nb += 1;

            if forward_ip > mflimit_plus_one {
                return emit_last_literals_with_base(&full, dst, base, anchor, op);
            }

            ref_pos = table[h];
            forward_h = hash4(&full, forward_ip);
            table[h] = ip;

            if ref_pos != usize::MAX
                && ip > ref_pos
                && ip - ref_pos <= LZ4_DISTANCE_MAX
                && full[ref_pos..ref_pos + MINMATCH] == full[ip..ip + MINMATCH]
            {
                break;
            }
        }

        while ip > anchor && ref_pos > 0 && full[ip - 1] == full[ref_pos - 1] {
            ip -= 1;
            ref_pos -= 1;
        }

        loop {
            let match_len =
                MINMATCH + count_match(&full, ip + MINMATCH, ref_pos + MINMATCH, match_limit);
            op =
                encode_sequence_with_base(src, dst, base, anchor, ip, match_len, ip - ref_pos, op)?;
            ip += match_len;
            anchor = ip;

            if ip >= mflimit_plus_one {
                return emit_last_literals_with_base(&full, dst, base, anchor, op);
            }

            table[hash4(&full, ip - 2)] = ip - 2;
            let h = hash4(&full, ip);
            ref_pos = table[h];
            table[h] = ip;
            if ref_pos != usize::MAX
                && ip > ref_pos
                && ip - ref_pos <= LZ4_DISTANCE_MAX
                && full[ref_pos..ref_pos + MINMATCH] == full[ip..ip + MINMATCH]
            {
                continue;
            }

            ip += 1;
            forward_h = hash4(&full, ip);
            break;
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct HcMatch {
    start: usize,
    len: usize,
    off: usize,
}

#[derive(Clone, Copy, Debug)]
struct HcOptimal {
    price: usize,
    off: usize,
    mlen: usize,
    litlen: usize,
}

#[derive(Debug)]
struct HcTables {
    hash: Vec<usize>,
    chain: Vec<usize>,
    next_to_update: usize,
}

impl HcTables {
    fn new(src_len: usize) -> Self {
        Self {
            hash: vec![usize::MAX; LZ4HC_HASH_SIZE],
            chain: vec![usize::MAX; cmp::min(src_len, LZ4_DISTANCE_MAX + 1)],
            next_to_update: 0,
        }
    }

    fn insert_until(&mut self, src: &[u8], target: usize) {
        let end = cmp::min(target, src.len().saturating_sub(MINMATCH - 1));
        while self.next_to_update < end {
            let pos = self.next_to_update;
            let h = hash4_hc(src, pos);
            if !self.chain.is_empty() {
                self.chain[pos & LZ4_DISTANCE_MAX] = self.hash[h];
            }
            self.hash[h] = pos;
            self.next_to_update += 1;
        }
    }

    fn previous(&self, pos: usize) -> usize {
        if self.chain.is_empty() {
            usize::MAX
        } else {
            self.chain[pos & LZ4_DISTANCE_MAX]
        }
    }
}

fn compress_block_hc(src: &[u8], dst: &mut [u8], compression_level: c_int) -> Option<usize> {
    if src.is_empty() {
        return emit_last_literals(src, dst, 0, 0);
    }
    if src.len() < MFLIMIT + 1 {
        return emit_last_literals(src, dst, 0, 0);
    }

    let level = normalize_hc_level(compression_level);
    if level >= 10 {
        return compress_block_hc_optimal(src, dst, 0, level);
    }

    let attempts = hc_search_attempts(compression_level);
    let mut table = HcTables::new(src.len());
    let mut anchor = 0usize;
    let mut op = 0usize;
    let mflimit = src.len() - MFLIMIT;
    let match_limit = src.len() - LAST_LITERALS;
    let nomatch = HcMatch {
        start: 0,
        len: 0,
        off: 0,
    };
    let mut ip = 0usize;

    while ip <= mflimit {
        let mut m1 = find_hc_match(src, &mut table, ip, match_limit, attempts);
        if m1.len < MINMATCH {
            ip += 1;
            continue;
        }
        let mut start0 = ip;
        let mut m0 = m1;

        'search2: loop {
            let mut start2;
            let mut m2;
            if ip + m1.len <= mflimit {
                start2 = ip + m1.len - 2;
                m2 =
                    find_hc_wider_match(src, &mut table, start2, ip, match_limit, m1.len, attempts);
                start2 = m2.start;
            } else {
                m2 = nomatch;
                start2 = 0;
            }

            if m2.len <= m1.len {
                op = encode_sequence(src, dst, anchor, ip, m1.len, m1.off, op)?;
                ip += m1.len;
                anchor = ip;
                break 'search2;
            }

            if start0 < ip && start2 < ip + m0.len {
                ip = start0;
                m1 = m0;
                continue 'search2;
            }

            if start2 - ip < 3 {
                ip = start2;
                m1 = m2;
                start0 = ip;
                m0 = m1;
                continue 'search2;
            }

            'search3: loop {
                if start2 - ip < OPTIMAL_ML {
                    let mut new_len = cmp::min(m1.len, OPTIMAL_ML);
                    if ip + new_len > start2 + m2.len - MINMATCH {
                        new_len = start2 - ip + m2.len - MINMATCH;
                    }
                    let correction = new_len.saturating_sub(start2 - ip);
                    if correction > 0 {
                        start2 += correction;
                        m2.start = start2;
                        m2.len -= correction;
                    }
                }

                let mut start3;
                let m3;
                if start2 + m2.len <= mflimit {
                    start3 = start2 + m2.len - 3;
                    m3 = find_hc_wider_match(
                        src,
                        &mut table,
                        start3,
                        start2,
                        match_limit,
                        m2.len,
                        attempts,
                    );
                    start3 = m3.start;
                } else {
                    m3 = nomatch;
                    start3 = 0;
                }

                if m3.len <= m2.len {
                    if start2 < ip + m1.len {
                        m1.len = start2 - ip;
                    }
                    op = encode_sequence(src, dst, anchor, ip, m1.len, m1.off, op)?;
                    ip += m1.len;
                    anchor = ip;

                    ip = start2;
                    op = encode_sequence(src, dst, anchor, ip, m2.len, m2.off, op)?;
                    ip += m2.len;
                    anchor = ip;
                    break 'search2;
                }

                if start3 < ip + m1.len + 3 {
                    if start3 >= ip + m1.len {
                        if start2 < ip + m1.len {
                            let correction = ip + m1.len - start2;
                            start2 += correction;
                            m2.start = start2;
                            m2.len = m2.len.saturating_sub(correction);
                            if m2.len < MINMATCH {
                                start2 = start3;
                                m2 = m3;
                            }
                        }
                        op = encode_sequence(src, dst, anchor, ip, m1.len, m1.off, op)?;
                        ip += m1.len;
                        anchor = ip;

                        ip = start3;
                        m1 = m3;
                        start0 = start2;
                        m0 = m2;
                        continue 'search2;
                    }

                    start2 = start3;
                    m2 = m3;
                    continue 'search3;
                }

                if start2 < ip + m1.len {
                    if start2 - ip < OPTIMAL_ML {
                        if m1.len > OPTIMAL_ML {
                            m1.len = OPTIMAL_ML;
                        }
                        if ip + m1.len > start2 + m2.len - MINMATCH {
                            m1.len = start2 - ip + m2.len - MINMATCH;
                        }
                        let correction = m1.len.saturating_sub(start2 - ip);
                        if correction > 0 {
                            start2 += correction;
                            m2.start = start2;
                            m2.len -= correction;
                        }
                    } else {
                        m1.len = start2 - ip;
                    }
                }

                op = encode_sequence(src, dst, anchor, ip, m1.len, m1.off, op)?;
                ip += m1.len;
                anchor = ip;

                ip = start2;
                m1 = m2;
                start2 = start3;
                m2 = m3;
                continue 'search3;
            }
        }
    }

    emit_last_literals(src, dst, anchor, op)
}

fn compress_block_hc_with_dict(
    src: &[u8],
    dst: &mut [u8],
    dict: &[u8],
    compression_level: c_int,
) -> Option<usize> {
    if src.is_empty() {
        return emit_last_literals(src, dst, 0, 0);
    }
    let dict_keep = cmp::min(dict.len(), LZ4_DISTANCE_MAX);
    let dict = &dict[dict.len() - dict_keep..];
    if dict.is_empty() || src.len() < MFLIMIT + 1 {
        return compress_block_hc(src, dst, compression_level);
    }

    let mut full = Vec::with_capacity(dict.len() + src.len());
    full.extend_from_slice(dict);
    full.extend_from_slice(src);
    let base = dict.len();
    let level = normalize_hc_level(compression_level);
    if level >= 10 {
        return compress_block_hc_optimal(&full, dst, base, level);
    }
    let attempts = hc_search_attempts(compression_level);
    let mut table = HcTables::new(full.len());
    table.insert_until(&full, base);

    let mut anchor = base;
    let mut op = 0usize;
    let mflimit = base + src.len() - MFLIMIT;
    let match_limit = base + src.len() - LAST_LITERALS;
    let mut ip = base;

    while ip <= mflimit {
        let mut m = find_hc_wider_match(
            &full,
            &mut table,
            ip,
            ip,
            match_limit,
            MINMATCH - 1,
            attempts,
        );
        if m.len < MINMATCH {
            ip += 1;
            continue;
        }

        if ip + m.len <= mflimit {
            let start2 = ip + m.len - 2;
            let m2 =
                find_hc_wider_match(&full, &mut table, start2, ip, match_limit, m.len, attempts);
            if m2.len > m.len && m2.start - ip >= 3 {
                if m2.start < ip + m.len {
                    m.len = m2.start - ip;
                }
            } else if m2.len > m.len && m2.start - ip < 3 {
                ip = m2.start;
                m = m2;
            }
        }

        op = encode_sequence_with_base(src, dst, base, anchor, ip, m.len, m.off, op)?;
        ip += m.len;
        anchor = ip;
    }

    emit_last_literals(src, dst, anchor - base, op)
}

fn compress_block_hc_optimal(
    full: &[u8],
    dst: &mut [u8],
    base: usize,
    compression_level: c_int,
) -> Option<usize> {
    let src_len = full.len().checked_sub(base)?;
    if src_len == 0 {
        return emit_last_literals_with_base(full, dst, base, base, 0);
    }
    if src_len < MFLIMIT + 1 {
        return emit_last_literals_with_base(full, dst, base, base, 0);
    }

    let attempts = hc_search_attempts(compression_level);
    let sufficient_len = hc_target_length(compression_level).min(LZ4_OPT_NUM - 1);
    let full_update = normalize_hc_level(compression_level) >= LZ4HC_CLEVEL_MAX;
    let mut table = HcTables::new(full.len());
    if base > 0 {
        table.insert_until(full, base);
    }

    let mut opt = vec![
        HcOptimal {
            price: usize::MAX / 4,
            off: 0,
            mlen: 1,
            litlen: 0,
        };
        LZ4_OPT_NUM + 3
    ];

    let mut ip = base;
    let mut anchor = base;
    let mut op = 0usize;
    let iend = full.len();
    let mflimit = iend - MFLIMIT;
    let match_limit = iend - LAST_LITERALS;

    while ip <= mflimit {
        let llen = ip - anchor;
        let first_match =
            find_hc_longer_match(full, &mut table, ip, match_limit, MINMATCH - 1, attempts);
        if first_match.len == 0 {
            ip += 1;
            continue;
        }

        if first_match.len > sufficient_len {
            op = encode_sequence_with_base(
                full,
                dst,
                base,
                anchor,
                ip,
                first_match.len,
                first_match.off,
                op,
            )?;
            ip += first_match.len;
            anchor = ip;
            continue;
        }

        opt.fill(HcOptimal {
            price: usize::MAX / 4,
            off: 0,
            mlen: 1,
            litlen: 0,
        });

        for rpos in 0..MINMATCH {
            opt[rpos] = HcOptimal {
                price: hc_literals_price(llen + rpos),
                off: 0,
                mlen: 1,
                litlen: llen + rpos,
            };
        }

        let first_len = first_match.len.min(LZ4_OPT_NUM - 4);
        for mlen in MINMATCH..=first_len {
            opt[mlen] = HcOptimal {
                price: hc_sequence_price(llen, mlen),
                off: first_match.off,
                mlen,
                litlen: llen,
            };
        }
        let mut last_match_pos = first_len;
        for add_lit in 1..=3 {
            let pos = last_match_pos + add_lit;
            opt[pos] = HcOptimal {
                price: opt[last_match_pos].price + hc_literals_price(add_lit),
                off: 0,
                mlen: 1,
                litlen: add_lit,
            };
        }

        let mut best_mlen = opt[last_match_pos].mlen;
        let mut best_off = opt[last_match_pos].off;
        let mut cur = 1usize;
        let mut immediate = false;

        while cur < last_match_pos {
            let cur_ptr = ip + cur;
            if cur_ptr > mflimit {
                break;
            }

            if full_update {
                if opt[cur + 1].price <= opt[cur].price
                    && opt[cur + MINMATCH].price < opt[cur].price + 3
                {
                    cur += 1;
                    continue;
                }
            } else if opt[cur + 1].price <= opt[cur].price {
                cur += 1;
                continue;
            }

            let min_len = if full_update {
                MINMATCH - 1
            } else {
                last_match_pos - cur
            };
            let new_match =
                find_hc_longer_match(full, &mut table, cur_ptr, match_limit, min_len, attempts);
            if new_match.len == 0 {
                cur += 1;
                continue;
            }

            if new_match.len > sufficient_len || new_match.len + cur >= LZ4_OPT_NUM {
                best_mlen = new_match.len;
                best_off = new_match.off;
                last_match_pos = cur + 1;
                immediate = true;
                break;
            }

            let base_litlen = opt[cur].litlen;
            for litlen in 1..MINMATCH {
                let pos = cur + litlen;
                let price = opt[cur]
                    .price
                    .saturating_sub(hc_literals_price(base_litlen))
                    + hc_literals_price(base_litlen + litlen);
                if price < opt[pos].price {
                    opt[pos] = HcOptimal {
                        price,
                        off: 0,
                        mlen: 1,
                        litlen: base_litlen + litlen,
                    };
                }
            }

            let match_len = new_match.len.min(LZ4_OPT_NUM - cur - 1);
            for ml in MINMATCH..=match_len {
                let pos = cur + ml;
                let (ll, price) = if opt[cur].mlen == 1 {
                    let ll = opt[cur].litlen;
                    let prefix = if cur > ll { opt[cur - ll].price } else { 0 };
                    (ll, prefix + hc_sequence_price(ll, ml))
                } else {
                    (0, opt[cur].price + hc_sequence_price(0, ml))
                };

                if pos > last_match_pos + 3 || price <= opt[pos].price {
                    if ml == match_len && last_match_pos < pos {
                        last_match_pos = pos;
                    }
                    opt[pos] = HcOptimal {
                        price,
                        off: new_match.off,
                        mlen: ml,
                        litlen: ll,
                    };
                }
            }

            for add_lit in 1..=3 {
                let pos = last_match_pos + add_lit;
                opt[pos] = HcOptimal {
                    price: opt[last_match_pos].price + hc_literals_price(add_lit),
                    off: 0,
                    mlen: 1,
                    litlen: add_lit,
                };
            }
            cur += 1;
        }

        if !immediate {
            best_mlen = opt[last_match_pos].mlen;
            best_off = opt[last_match_pos].off;
            cur = last_match_pos.saturating_sub(best_mlen);
        }

        let mut candidate_pos = cur;
        let mut selected_match_length = best_mlen;
        let mut selected_offset = best_off;
        loop {
            let next_match_length = opt[candidate_pos].mlen;
            let next_offset = opt[candidate_pos].off;
            opt[candidate_pos].mlen = selected_match_length;
            opt[candidate_pos].off = selected_offset;
            selected_match_length = next_match_length;
            selected_offset = next_offset;
            if next_match_length > candidate_pos {
                break;
            }
            candidate_pos -= next_match_length;
        }

        let mut rpos = 0usize;
        while rpos < last_match_pos {
            let ml = opt[rpos].mlen;
            let offset = opt[rpos].off;
            if ml == 1 {
                ip += 1;
                rpos += 1;
                continue;
            }
            rpos += ml;
            op = encode_sequence_with_base(&full[base..], dst, base, anchor, ip, ml, offset, op)?;
            ip += ml;
            anchor = ip;
        }
    }

    emit_last_literals_with_base(full, dst, base, anchor, op)
}

fn compress_frame_block(
    src: &[u8],
    dst: &mut [u8],
    prefs: &FramePrefs,
    dict: &[u8],
) -> Option<usize> {
    if prefs.compression_level > 0 && !dict.is_empty() {
        compress_block_hc_with_dict(src, dst, dict, prefs.compression_level)
    } else if prefs.compression_level > 0 {
        compress_block_hc(src, dst, prefs.compression_level)
    } else {
        compress_block(src, dst)
    }
}

fn normalize_hc_level(compression_level: c_int) -> c_int {
    if compression_level < 1 {
        LZ4HC_CLEVEL_DEFAULT
    } else {
        cmp::min(compression_level, LZ4HC_CLEVEL_MAX)
    }
}

fn hc_search_attempts(compression_level: c_int) -> usize {
    let level = normalize_hc_level(compression_level);
    match level {
        0..=2 => 2,
        3 => 4,
        4 => 8,
        5 => 16,
        6 => 32,
        7 => 64,
        8 => 128,
        9 => 256,
        10 => 96,
        11 => 512,
        _ => 16_384,
    }
}

fn hc_target_length(compression_level: c_int) -> usize {
    match normalize_hc_level(compression_level) {
        10 => 64,
        11 => 128,
        12 => LZ4_OPT_NUM,
        _ => 16,
    }
}

fn hc_literals_price(lit_len: usize) -> usize {
    let mut price = lit_len;
    if lit_len >= 15 {
        price += 1 + ((lit_len - 15) / 255);
    }
    price
}

fn hc_sequence_price(lit_len: usize, match_len: usize) -> usize {
    let mut price = 1 + 2 + hc_literals_price(lit_len);
    if match_len >= 15 + MINMATCH {
        price += 1 + ((match_len - (15 + MINMATCH)) / 255);
    }
    price
}

fn compress_hc_dest_size(
    src: &[u8],
    dst: &mut [u8],
    compression_level: c_int,
) -> Option<(usize, usize)> {
    if src.is_empty() {
        let written = compress_block_hc(src, dst, compression_level)?;
        return Some((0, written));
    }

    let mut low = 0usize;
    let mut high = src.len();
    let mut best: Option<(usize, usize, Vec<u8>)> = None;
    while low <= high {
        let mid = low + (high - low) / 2;
        let mut candidate = vec![0u8; dst.len()];
        match compress_block_hc(&src[..mid], &mut candidate, compression_level) {
            Some(written) if written <= dst.len() => {
                best = Some((mid, written, candidate));
                low = mid + 1;
            }
            _ => {
                if mid == 0 {
                    break;
                }
                high = mid - 1;
            }
        }
    }

    let (consumed, written, candidate) = best?;
    dst[..written].copy_from_slice(&candidate[..written]);
    Some((consumed, written))
}

fn compress_hc_dest_size_with_dict(
    src: &[u8],
    dst: &mut [u8],
    dict: &[u8],
    compression_level: c_int,
) -> Option<(usize, usize)> {
    if src.is_empty() {
        let written = compress_block_hc_with_dict(src, dst, dict, compression_level)?;
        return Some((0, written));
    }

    let mut low = 0usize;
    let mut high = src.len();
    let mut best: Option<(usize, usize, Vec<u8>)> = None;
    while low <= high {
        let mid = low + (high - low) / 2;
        let mut candidate = vec![0u8; dst.len()];
        match compress_block_hc_with_dict(&src[..mid], &mut candidate, dict, compression_level) {
            Some(written) if written <= dst.len() => {
                best = Some((mid, written, candidate));
                low = mid + 1;
            }
            _ => {
                if mid == 0 {
                    break;
                }
                high = mid - 1;
            }
        }
    }

    let (consumed, written, candidate) = best?;
    dst[..written].copy_from_slice(&candidate[..written]);
    Some((consumed, written))
}

fn append_hc_dictionary(dictionary: &mut Vec<u8>, src: &[u8]) {
    dictionary.extend_from_slice(src);
    if dictionary.len() > LZ4_DISTANCE_MAX {
        let drop_len = dictionary.len() - LZ4_DISTANCE_MAX;
        dictionary.drain(..drop_len);
    }
}

fn find_hc_match(
    src: &[u8],
    table: &mut HcTables,
    ip: usize,
    match_limit: usize,
    max_attempts: usize,
) -> HcMatch {
    find_hc_wider_match(src, table, ip, ip, match_limit, MINMATCH - 1, max_attempts)
}

fn find_hc_longer_match(
    src: &[u8],
    table: &mut HcTables,
    ip: usize,
    match_limit: usize,
    min_len: usize,
    max_attempts: usize,
) -> HcMatch {
    let m = find_hc_wider_match(src, table, ip, ip, match_limit, min_len, max_attempts);
    if m.len <= min_len {
        HcMatch {
            start: ip,
            len: 0,
            off: 0,
        }
    } else {
        m
    }
}

fn find_hc_wider_match(
    src: &[u8],
    table: &mut HcTables,
    ip: usize,
    low_limit: usize,
    match_limit: usize,
    longest: usize,
    max_attempts: usize,
) -> HcMatch {
    table.insert_until(src, ip);
    if ip + MINMATCH > match_limit {
        return HcMatch {
            start: ip,
            len: 0,
            off: 0,
        };
    }

    let mut candidate = table.hash[hash4_hc(src, ip)];
    let mut attempts = max_attempts;
    let lowest = ip.saturating_sub(LZ4_DISTANCE_MAX);
    let mut best = HcMatch {
        start: ip,
        len: longest,
        off: 0,
    };

    while candidate != usize::MAX && candidate >= lowest && candidate < ip && attempts > 0 {
        attempts -= 1;
        if candidate + MINMATCH <= src.len()
            && src[candidate..candidate + MINMATCH] == src[ip..ip + MINMATCH]
        {
            let forward =
                MINMATCH + count_match(src, ip + MINMATCH, candidate + MINMATCH, match_limit);
            let back = count_back(src, ip, candidate, low_limit);
            let len = forward + back;
            if len > best.len {
                best = HcMatch {
                    start: ip - back,
                    len,
                    off: ip - candidate,
                };
                if ip + forward == match_limit {
                    break;
                }
            }
        }
        candidate = table.previous(candidate);
    }

    best
}

fn count_back(src: &[u8], ip: usize, candidate: usize, low_limit: usize) -> usize {
    let mut back = 0usize;
    while ip > low_limit + back
        && candidate > back
        && src[ip - back - 1] == src[candidate - back - 1]
    {
        back += 1;
    }
    back
}

fn count_match(src: &[u8], mut ip: usize, mut match_pos: usize, limit: usize) -> usize {
    let start = ip;
    while ip < limit && src[ip] == src[match_pos] {
        ip += 1;
        match_pos += 1;
    }
    ip - start
}

fn encode_sequence(
    src: &[u8],
    dst: &mut [u8],
    anchor: usize,
    ip: usize,
    match_len: usize,
    offset: usize,
    mut op: usize,
) -> Option<usize> {
    if offset == 0 || offset > LZ4_DISTANCE_MAX {
        return None;
    }
    let lit_len = ip - anchor;
    let token_pos = op;
    if op >= dst.len() {
        return None;
    }
    op += 1;
    op = emit_len(dst, op, lit_len, 15)?;
    if op + lit_len + 2 > dst.len() {
        return None;
    }
    dst[op..op + lit_len].copy_from_slice(&src[anchor..ip]);
    op += lit_len;
    dst[op..op + 2].copy_from_slice(&(offset as u16).to_le_bytes());
    op += 2;

    let ml_code = match_len - MINMATCH;
    dst[token_pos] = ((cmp::min(lit_len, 15) as u8) << 4) | cmp::min(ml_code, 15) as u8;
    emit_len(dst, op, ml_code, 15)
}

fn encode_sequence_with_base(
    src: &[u8],
    dst: &mut [u8],
    base: usize,
    anchor: usize,
    ip: usize,
    match_len: usize,
    offset: usize,
    mut op: usize,
) -> Option<usize> {
    if anchor < base
        || ip < anchor
        || ip - base > src.len()
        || offset == 0
        || offset > LZ4_DISTANCE_MAX
    {
        return None;
    }
    let lit_len = ip - anchor;
    let literal_start = anchor - base;
    let token_pos = op;
    if op >= dst.len() {
        return None;
    }
    op += 1;
    op = emit_len(dst, op, lit_len, 15)?;
    if op + lit_len + 2 > dst.len() {
        return None;
    }
    dst[op..op + lit_len].copy_from_slice(&src[literal_start..literal_start + lit_len]);
    op += lit_len;
    dst[op..op + 2].copy_from_slice(&(offset as u16).to_le_bytes());
    op += 2;

    let ml_code = match_len - MINMATCH;
    dst[token_pos] = ((cmp::min(lit_len, 15) as u8) << 4) | cmp::min(ml_code, 15) as u8;
    emit_len(dst, op, ml_code, 15)
}

fn emit_last_literals(src: &[u8], dst: &mut [u8], anchor: usize, mut op: usize) -> Option<usize> {
    let lit_len = src.len() - anchor;
    if op >= dst.len() {
        return None;
    }
    let token_pos = op;
    op += 1;
    dst[token_pos] = (cmp::min(lit_len, 15) as u8) << 4;
    op = emit_len(dst, op, lit_len, 15)?;
    if op + lit_len > dst.len() {
        return None;
    }
    dst[op..op + lit_len].copy_from_slice(&src[anchor..]);
    Some(op + lit_len)
}

fn emit_last_literals_with_base(
    full: &[u8],
    dst: &mut [u8],
    base: usize,
    anchor: usize,
    mut op: usize,
) -> Option<usize> {
    if anchor < base || anchor > full.len() {
        return None;
    }
    let lit_len = full.len() - anchor;
    if op >= dst.len() {
        return None;
    }
    let token_pos = op;
    op += 1;
    dst[token_pos] = (cmp::min(lit_len, 15) as u8) << 4;
    op = emit_len(dst, op, lit_len, 15)?;
    if op + lit_len > dst.len() {
        return None;
    }
    let literal_start = anchor - base;
    dst[op..op + lit_len]
        .copy_from_slice(&full[base + literal_start..base + literal_start + lit_len]);
    Some(op + lit_len)
}

fn emit_len(dst: &mut [u8], mut op: usize, len: usize, base: usize) -> Option<usize> {
    if len >= base {
        let mut extra = len - base;
        while extra >= 255 {
            if op >= dst.len() {
                return None;
            }
            dst[op] = 255;
            op += 1;
            extra -= 255;
        }
        if op >= dst.len() {
            return None;
        }
        dst[op] = extra as u8;
        op += 1;
    }
    Some(op)
}

fn decompress_block(src: &[u8], dst: &mut [u8]) -> Option<usize> {
    decompress_block_with_dict(src, dst, &[])
}

fn decompress_block_with_dict(src: &[u8], dst: &mut [u8], dict: &[u8]) -> Option<usize> {
    let mut ip = 0usize;
    let mut op = 0usize;
    while ip < src.len() {
        let token = src[ip];
        ip += 1;

        let lit_len = read_len(src, &mut ip, (token >> 4) as usize)?;
        if ip + lit_len > src.len() || op + lit_len > dst.len() {
            return None;
        }
        dst[op..op + lit_len].copy_from_slice(&src[ip..ip + lit_len]);
        ip += lit_len;
        op += lit_len;
        if ip == src.len() {
            return Some(op);
        }
        if ip + 2 > src.len() {
            return None;
        }
        let offset = u16::from_le_bytes([src[ip], src[ip + 1]]) as usize;
        ip += 2;
        if offset == 0 || offset > op + dict.len() {
            return None;
        }
        let match_len = read_len(src, &mut ip, (token & 0x0f) as usize)? + MINMATCH;
        if op + match_len > dst.len() {
            return None;
        }
        copy_match(dst, dict, &mut op, offset, match_len)?;
    }
    Some(op)
}

fn decompress_block_partial(src: &[u8], dst: &mut [u8], target: usize) -> Option<usize> {
    decompress_block_partial_with_dict(src, dst, target, &[])
}

fn decompress_block_partial_with_dict(
    src: &[u8],
    dst: &mut [u8],
    target: usize,
    dict: &[u8],
) -> Option<usize> {
    if target == 0 {
        return Some(0);
    }
    if target > dst.len() {
        return None;
    }

    let mut ip = 0usize;
    let mut op = 0usize;
    while ip < src.len() && op < target {
        let token = src[ip];
        ip += 1;

        let lit_len = read_len(src, &mut ip, (token >> 4) as usize)?;
        if ip + lit_len > src.len() || op + lit_len > dst.len() {
            return None;
        }
        let lit_copy = cmp::min(lit_len, target - op);
        dst[op..op + lit_copy].copy_from_slice(&src[ip..ip + lit_copy]);
        op += lit_copy;
        if op == target {
            return Some(op);
        }
        ip += lit_len;
        if ip == src.len() {
            return Some(op);
        }
        if ip + 2 > src.len() {
            return None;
        }
        let offset = u16::from_le_bytes([src[ip], src[ip + 1]]) as usize;
        ip += 2;
        if offset == 0 || offset > op + dict.len() {
            return None;
        }
        let match_len = read_len(src, &mut ip, (token & 0x0f) as usize)? + MINMATCH;
        if op + match_len > dst.len() {
            return None;
        }
        let match_copy = cmp::min(match_len, target - op);
        copy_match(dst, dict, &mut op, offset, match_copy)?;
    }
    Some(op)
}

fn decompress_block_exact_ptr(src: *const u8, dst: &mut [u8], dict: &[u8]) -> Option<usize> {
    let mut ip = 0usize;
    let mut op = 0usize;
    while op < dst.len() {
        let token = unsafe { *src.add(ip) };
        ip += 1;

        let lit_len = read_len_ptr(src, &mut ip, (token >> 4) as usize)?;
        if op + lit_len > dst.len() {
            return None;
        }
        unsafe {
            ptr::copy_nonoverlapping(src.add(ip), dst.as_mut_ptr().add(op), lit_len);
        }
        ip += lit_len;
        op += lit_len;
        if op == dst.len() {
            return Some(ip);
        }

        let offset = unsafe { u16::from_le_bytes([*src.add(ip), *src.add(ip + 1)]) as usize };
        ip += 2;
        if offset == 0 || offset > op + dict.len() {
            return None;
        }
        let match_len = read_len_ptr(src, &mut ip, (token & 0x0f) as usize)? + MINMATCH;
        if op + match_len > dst.len() {
            return None;
        }
        copy_match(dst, dict, &mut op, offset, match_len)?;
    }
    Some(ip)
}

fn read_len_ptr(src: *const u8, ip: &mut usize, mut len: usize) -> Option<usize> {
    if len == 15 {
        loop {
            let b = unsafe { *src.add(*ip) } as usize;
            *ip += 1;
            len = len.checked_add(b)?;
            if b != 255 {
                break;
            }
        }
    }
    Some(len)
}

fn copy_match(
    dst: &mut [u8],
    dict: &[u8],
    op: &mut usize,
    offset: usize,
    mut len: usize,
) -> Option<()> {
    if offset == 0 {
        return None;
    }
    if offset > *op {
        let dict_offset = offset - *op;
        if dict_offset > dict.len() {
            return None;
        }
        let dict_pos = dict.len() - dict_offset;
        let dict_len = cmp::min(len, dict.len() - dict_pos);
        if *op + dict_len > dst.len() {
            return None;
        }
        dst[*op..*op + dict_len].copy_from_slice(&dict[dict_pos..dict_pos + dict_len]);
        *op += dict_len;
        len -= dict_len;
    }

    if len == 0 {
        return Some(());
    }
    if offset > *op {
        return None;
    }
    if *op + len > dst.len() {
        return None;
    }

    let first = cmp::min(offset, len);
    let src = *op - offset;
    if src + first > *op {
        return None;
    }
    dst.copy_within(src..src + first, *op);
    *op += first;
    len -= first;

    let mut copied = first;
    while len > 0 {
        let chunk = cmp::min(copied, len);
        let src = *op - copied;
        dst.copy_within(src..src + chunk, *op);
        *op += chunk;
        len -= chunk;
        copied += chunk;
    }
    Some(())
}

fn read_len(src: &[u8], ip: &mut usize, mut len: usize) -> Option<usize> {
    if len == 15 {
        loop {
            if *ip >= src.len() {
                return None;
            }
            let b = src[*ip] as usize;
            *ip += 1;
            len = len.checked_add(b)?;
            if b != 255 {
                break;
            }
        }
    }
    Some(len)
}

fn hash4(src: &[u8], pos: usize) -> usize {
    let v = u32::from_le_bytes([src[pos], src[pos + 1], src[pos + 2], src[pos + 3]]);
    ((v.wrapping_mul(2_654_435_761)) >> (32 - HASH_BITS)) as usize
}

fn hash4_hc(src: &[u8], pos: usize) -> usize {
    let v = u32::from_le_bytes([src[pos], src[pos + 1], src[pos + 2], src[pos + 3]]);
    ((v.wrapping_mul(2_654_435_761)) >> (32 - LZ4HC_HASH_BITS)) as usize
}

fn preferences_from_ptr(ptr: *const LZ4FPreferences) -> FramePrefs {
    if ptr.is_null() {
        return FramePrefs::default();
    }
    unsafe {
        let prefs = &*ptr;
        let id = match prefs.frame_info.block_size_id {
            BlockSize::Default | BlockSize::Max64KB => 4,
            BlockSize::Max256KB => 5,
            BlockSize::Max1MB => 6,
            BlockSize::Max4MB => 7,
        };
        FramePrefs {
            block_size_id: id,
            block_independent: matches!(prefs.frame_info.block_mode, BlockMode::Independent),
            block_checksum: matches!(
                prefs.frame_info.block_checksum_flag,
                BlockChecksum::BlockChecksumEnabled
            ),
            content_checksum: matches!(
                prefs.frame_info.content_checksum_flag,
                ContentChecksum::ChecksumEnabled
            ),
            content_size: prefs.frame_info.content_size,
            compression_level: prefs.compression_level as c_int,
        }
    }
}

fn frame_header(prefs: FramePrefs) -> Vec<u8> {
    let mut out = Vec::with_capacity(19);
    out.extend_from_slice(&LZ4F_MAGIC);
    let mut flg = 0x40;
    if prefs.block_independent {
        flg |= 0x20;
    }
    if prefs.block_checksum {
        flg |= 0x10;
    }
    if prefs.content_size != 0 {
        flg |= 0x08;
    }
    if prefs.content_checksum {
        flg |= 0x04;
    }
    out.push(flg);
    out.push(prefs.block_size_id << 4);
    if prefs.content_size != 0 {
        out.extend_from_slice(&prefs.content_size.to_le_bytes());
    }
    let hc = (xxhash32(&out[4..], 0) >> 8) as u8;
    out.push(hc);
    out
}

fn parse_frame_header(src: &[u8]) -> Option<(FramePrefs, usize)> {
    if src.len() < 7 || src[..4] != LZ4F_MAGIC {
        return None;
    }
    let flg = src[4];
    if flg & 0xC0 != 0x40 {
        return None;
    }
    let bd = src[5];
    let block_size_id = (bd >> 4) & 0x07;
    if !(4..=7).contains(&block_size_id) {
        return None;
    }
    let mut pos = 6;
    let mut content_size = 0u64;
    if flg & 0x08 != 0 {
        if src.len() < pos + 8 + 1 {
            return None;
        }
        content_size = u64::from_le_bytes(src[pos..pos + 8].try_into().ok()?);
        pos += 8;
    }
    if flg & 0x01 != 0 {
        if src.len() < pos + 4 + 1 {
            return None;
        }
        pos += 4;
    }
    if src.len() < pos + 1 {
        return None;
    }
    let expected_hc = (xxhash32(&src[4..pos], 0) >> 8) as u8;
    if src[pos] != expected_hc {
        return None;
    }
    pos += 1;
    Some((
        FramePrefs {
            block_size_id,
            block_independent: flg & 0x20 != 0,
            block_checksum: flg & 0x10 != 0,
            content_checksum: flg & 0x04 != 0,
            content_size,
            compression_level: 0,
        },
        pos,
    ))
}

fn is_skippable_magic_prefix(src: &[u8]) -> bool {
    if src.len() < 4 {
        return false;
    }
    let magic = u32::from_le_bytes(src[..4].try_into().unwrap());
    (LZ4F_SKIPPABLE_MAGIC_MIN..=LZ4F_SKIPPABLE_MAGIC_MAX).contains(&magic)
}

fn parse_skippable_frame_len(src: &[u8]) -> Option<usize> {
    if src.len() < 8 || !is_skippable_magic_prefix(src) {
        return None;
    }
    let content_len = u32::from_le_bytes(src[4..8].try_into().ok()?) as usize;
    content_len.checked_add(8)
}

fn parse_frame_header_if_available(ctx: &mut DecompressionCtx) -> Result<bool, usize> {
    if ctx.done || ctx.parsed_header {
        return Ok(true);
    }
    if ctx.input.len().saturating_sub(ctx.pos) >= 4
        && is_skippable_magic_prefix(&ctx.input[ctx.pos..])
    {
        if ctx.input.len().saturating_sub(ctx.pos) < 8 {
            return Ok(false);
        }
        let Some(skip_len) = parse_skippable_frame_len(&ctx.input[ctx.pos..]) else {
            return Err(ERROR_BAD_HEADER);
        };
        if ctx.input.len().saturating_sub(ctx.pos) < skip_len {
            compact_input(ctx);
            return Ok(false);
        }
        ctx.pos += skip_len;
        ctx.done = true;
        compact_input(ctx);
        return Ok(true);
    }
    if ctx.input.len().saturating_sub(ctx.pos) < 7 {
        return Ok(false);
    }
    if let Some(header_len) = expected_frame_header_len(&ctx.input[ctx.pos..]) {
        if ctx.input.len().saturating_sub(ctx.pos) < header_len {
            return Ok(false);
        }
    }
    let Some((prefs, header_len)) = parse_frame_header(&ctx.input[ctx.pos..]) else {
        return Err(ERROR_BAD_HEADER);
    };
    ctx.pos += header_len;
    ctx.parsed_header = true;
    ctx.block_checksum = prefs.block_checksum;
    ctx.content_checksum = prefs.content_checksum;
    ctx.content_size = prefs.content_size;
    ctx.content_read = 0;
    ctx.block_independent = prefs.block_independent;
    ctx.block_max = block_max_size(prefs.block_size_id);
    if !ctx.external_dictionary {
        ctx.dictionary.clear();
    }
    ctx.external_dictionary = false;
    Ok(true)
}

fn try_decompress_frame_block_to_dst(
    ctx: &mut DecompressionCtx,
    dst: &mut [u8],
) -> Option<Result<usize, usize>> {
    if ctx.done
        || !ctx.parsed_header
        || !pending_is_empty(ctx)
    {
        return None;
    }
    if ctx.input.len().saturating_sub(ctx.pos) < 4 {
        compact_input(ctx);
        return None;
    }
    let block_header = u32::from_le_bytes(ctx.input[ctx.pos..ctx.pos + 4].try_into().unwrap());
    let raw = block_header & 0x8000_0000 != 0;
    let block_len = (block_header & 0x7FFF_FFFF) as usize;
    if block_len == 0 {
        return None;
    }
    let checksum_len = if ctx.block_checksum { 4 } else { 0 };
    if ctx.input.len().saturating_sub(ctx.pos) < 4 + block_len + checksum_len {
        compact_input(ctx);
        return None;
    }
    if raw {
        if dst.len() < block_len {
            return None;
        }
    } else if dst.len() < ctx.block_max {
        return None;
    }

    ctx.pos += 4;
    let block_start = ctx.pos;
    let block_end = block_start + block_len;
    if ctx.block_checksum {
        let stored = u32::from_le_bytes(
            ctx.input[block_end..block_end + checksum_len]
                .try_into()
                .unwrap(),
        );
        if stored != xxhash32(&ctx.input[block_start..block_end], 0) {
            return Some(Err(ERROR_CHECKSUM_INVALID));
        }
    }

    let written = if raw {
        dst[..block_len].copy_from_slice(&ctx.input[block_start..block_end]);
        block_len
    } else {
        let n = if ctx.block_independent && ctx.dictionary.is_empty() {
            decompress_block(&ctx.input[block_start..block_end], dst)
        } else {
            decompress_block_with_dict(&ctx.input[block_start..block_end], dst, &ctx.dictionary)
        };
        match n {
            Some(n) => n,
            None => return Some(Err(ERROR_GENERIC)),
        }
    };

    ctx.content_hasher.update(&dst[..written]);
    ctx.content_read += written as u64;
    if !ctx.block_independent {
        append_hc_dictionary(&mut ctx.dictionary, &dst[..written]);
    } else {
        ctx.dictionary.clear();
    }
    ctx.pos = block_end + checksum_len;
    if ctx.content_size != 0 && ctx.content_read > ctx.content_size {
        return Some(Err(ERROR_GENERIC));
    }
    if ctx.content_size != 0 {
        let trailer = if ctx.content_checksum { 4 } else { 0 };
        if ctx.input.len().saturating_sub(ctx.pos) >= 4 {
            let end_mark = u32::from_le_bytes(ctx.input[ctx.pos..ctx.pos + 4].try_into().unwrap());
            if end_mark != 0 {
                if ctx.content_read >= ctx.content_size {
                    return Some(Err(ERROR_GENERIC));
                }
            } else if ctx.content_read != ctx.content_size {
                return Some(Err(ERROR_GENERIC));
            } else if ctx.input.len().saturating_sub(ctx.pos) >= 4 + trailer {
                ctx.pos += 4;
                if ctx.content_checksum {
                    let stored =
                        u32::from_le_bytes(ctx.input[ctx.pos..ctx.pos + 4].try_into().unwrap());
                    if stored != ctx.content_hasher.digest() {
                        return Some(Err(ERROR_CHECKSUM_INVALID));
                    }
                    ctx.pos += 4;
                }
                ctx.done = true;
            }
        }
    }
    compact_input(ctx);
    Some(Ok(written))
}

fn parse_available_frame(ctx: &mut DecompressionCtx) -> Result<(), usize> {
    if ctx.done {
        return Ok(());
    }
    if !parse_frame_header_if_available(ctx)? || ctx.done {
        return Ok(());
    }

    loop {
        if ctx.input.len().saturating_sub(ctx.pos) < 4 {
            compact_input(ctx);
            return Ok(());
        }
        let block_header = u32::from_le_bytes(ctx.input[ctx.pos..ctx.pos + 4].try_into().unwrap());
        let raw = block_header & 0x8000_0000 != 0;
        let block_len = (block_header & 0x7FFF_FFFF) as usize;
        if block_len == 0 {
            let trailer = if ctx.content_checksum { 4 } else { 0 };
            if ctx.input.len().saturating_sub(ctx.pos) < 4 + trailer {
                compact_input(ctx);
                return Ok(());
            }
            ctx.pos += 4;
            if ctx.content_checksum {
                let stored =
                    u32::from_le_bytes(ctx.input[ctx.pos..ctx.pos + 4].try_into().unwrap());
                if stored != ctx.content_hasher.digest() {
                    return Err(ERROR_CHECKSUM_INVALID);
                }
                ctx.pos += 4;
            }
            if ctx.content_size != 0 && ctx.content_read != ctx.content_size {
                return Err(ERROR_GENERIC);
            }
            ctx.done = true;
            compact_input(ctx);
            return Ok(());
        }
        let checksum_len = if ctx.block_checksum { 4 } else { 0 };
        if ctx.input.len().saturating_sub(ctx.pos) < 4 + block_len + checksum_len {
            compact_input(ctx);
            return Ok(());
        }
        ctx.pos += 4;
        let block_start = ctx.pos;
        let block_end = block_start + block_len;
        if ctx.block_checksum {
            let stored = u32::from_le_bytes(
                ctx.input[block_end..block_end + checksum_len]
                    .try_into()
                    .unwrap(),
            );
            if stored != xxhash32(&ctx.input[block_start..block_end], 0) {
                return Err(ERROR_CHECKSUM_INVALID);
            }
        }
        if raw {
            ctx.content_hasher
                .update(&ctx.input[block_start..block_end]);
            ctx.content_read += block_len as u64;
            ctx.pending
                .extend_from_slice(&ctx.input[block_start..block_end]);
            if !ctx.block_independent {
                append_hc_dictionary(&mut ctx.dictionary, &ctx.input[block_start..block_end]);
            } else {
                ctx.dictionary.clear();
            }
            ctx.pos = block_end + checksum_len;
        } else {
            let mut out = vec![0u8; ctx.block_max];
            let n = if ctx.block_independent && ctx.dictionary.is_empty() {
                decompress_block(&ctx.input[block_start..block_end], &mut out)
            } else {
                decompress_block_with_dict(
                    &ctx.input[block_start..block_end],
                    &mut out,
                    &ctx.dictionary,
                )
            }
            .ok_or(ERROR_GENERIC)?;
            ctx.content_hasher.update(&out[..n]);
            ctx.content_read += n as u64;
            ctx.pending.extend_from_slice(&out[..n]);
            if !ctx.block_independent {
                append_hc_dictionary(&mut ctx.dictionary, &out[..n]);
            } else {
                ctx.dictionary.clear();
            }
            ctx.pos = block_end + checksum_len;
        }
        if ctx.content_size != 0 && ctx.content_read > ctx.content_size {
            return Err(ERROR_GENERIC);
        }
        if !pending_is_empty(ctx) {
            compact_input(ctx);
            return Ok(());
        }
    }
}

fn pending_len(ctx: &DecompressionCtx) -> usize {
    ctx.pending.len().saturating_sub(ctx.pending_pos)
}

fn pending_is_empty(ctx: &DecompressionCtx) -> bool {
    pending_len(ctx) == 0
}

fn compact_input(ctx: &mut DecompressionCtx) {
    if ctx.pos > 0 {
        ctx.input.drain(..ctx.pos);
        ctx.pos = 0;
    }
}

fn consumed_from_call(done: bool, src_size: usize, remaining_input_len: usize) -> usize {
    if !done {
        return src_size;
    }
    let remaining_from_call = cmp::min(remaining_input_len, src_size);
    src_size.saturating_sub(remaining_from_call)
}

fn expected_frame_header_len(src: &[u8]) -> Option<usize> {
    if src.len() < 6 || src[..4] != LZ4F_MAGIC {
        return None;
    }
    let flg = src[4];
    if flg & 0xC0 != 0x40 {
        return None;
    }
    let mut len = 7;
    if flg & 0x08 != 0 {
        len += 8;
    }
    if flg & 0x01 != 0 {
        len += 4;
    }
    Some(len)
}

fn frame_hint(ctx: &DecompressionCtx) -> usize {
    if !ctx.parsed_header {
        let available = ctx.input.len().saturating_sub(ctx.pos);
        let expected = if available >= 6 {
            expected_frame_header_len(&ctx.input[ctx.pos..]).unwrap_or(7)
        } else {
            7
        };
        expected.saturating_sub(available)
    } else if pending_is_empty(ctx) {
        let available = ctx.input.len().saturating_sub(ctx.pos);
        if available < 4 {
            return 4 - available;
        }
        let block_header = u32::from_le_bytes(ctx.input[ctx.pos..ctx.pos + 4].try_into().unwrap());
        let block_len = (block_header & 0x7FFF_FFFF) as usize;
        let needed = if block_len == 0 {
            4 + if ctx.content_checksum { 4 } else { 0 }
        } else {
            4 + block_len + if ctx.block_checksum { 4 } else { 0 }
        };
        cmp::max(needed.saturating_sub(available), 1)
    } else {
        1
    }
}

fn block_size_enum(id: u8) -> BlockSize {
    match id {
        5 => BlockSize::Max256KB,
        6 => BlockSize::Max1MB,
        7 => BlockSize::Max4MB,
        _ => BlockSize::Max64KB,
    }
}

fn block_max_size(id: u8) -> usize {
    match id {
        5 => 256 * 1024,
        6 => 1024 * 1024,
        7 => 4 * 1024 * 1024,
        _ => 64 * 1024,
    }
}

fn xxhash32(input: &[u8], seed: u32) -> u32 {
    let mut h = XxHash32::new(seed);
    h.update(input);
    h.digest()
}

#[derive(Clone, Copy, Debug)]
struct XxHash32 {
    total: usize,
    seed: u32,
    v1: u32,
    v2: u32,
    v3: u32,
    v4: u32,
    mem: [u8; 16],
    mem_len: usize,
}

impl XxHash32 {
    fn new(seed: u32) -> Self {
        Self {
            total: 0,
            seed,
            v1: seed.wrapping_add(0x9E37_79B1).wrapping_add(0x85EB_CA77),
            v2: seed.wrapping_add(0x85EB_CA77),
            v3: seed,
            v4: seed.wrapping_sub(0x9E37_79B1),
            mem: [0; 16],
            mem_len: 0,
        }
    }

    fn update(&mut self, mut input: &[u8]) {
        self.total += input.len();
        if self.mem_len + input.len() < 16 {
            self.mem[self.mem_len..self.mem_len + input.len()].copy_from_slice(input);
            self.mem_len += input.len();
            return;
        }
        if self.mem_len > 0 {
            let fill = 16 - self.mem_len;
            self.mem[self.mem_len..16].copy_from_slice(&input[..fill]);
            let block = self.mem;
            self.process(&block);
            input = &input[fill..];
            self.mem_len = 0;
        }
        while input.len() >= 16 {
            self.process(&input[..16]);
            input = &input[16..];
        }
        self.mem[..input.len()].copy_from_slice(input);
        self.mem_len = input.len();
    }

    fn digest(&self) -> u32 {
        let mut h = if self.total >= 16 {
            self.v1
                .rotate_left(1)
                .wrapping_add(self.v2.rotate_left(7))
                .wrapping_add(self.v3.rotate_left(12))
                .wrapping_add(self.v4.rotate_left(18))
        } else {
            self.seed.wrapping_add(0x1656_67B1)
        };
        h = h.wrapping_add(self.total as u32);
        let mut p = &self.mem[..self.mem_len];
        while p.len() >= 4 {
            h = h
                .wrapping_add(read_u32(p).wrapping_mul(0xC2B2_AE3D))
                .rotate_left(17)
                .wrapping_mul(0x27D4_EB2F);
            p = &p[4..];
        }
        for &b in p {
            h = h
                .wrapping_add((b as u32).wrapping_mul(0x1656_67B1))
                .rotate_left(11)
                .wrapping_mul(0x9E37_79B1);
        }
        h ^= h >> 15;
        h = h.wrapping_mul(0x85EB_CA77);
        h ^= h >> 13;
        h = h.wrapping_mul(0xC2B2_AE3D);
        h ^ (h >> 16)
    }

    fn process(&mut self, block: &[u8]) {
        self.v1 = round(self.v1, read_u32(&block[0..]));
        self.v2 = round(self.v2, read_u32(&block[4..]));
        self.v3 = round(self.v3, read_u32(&block[8..]));
        self.v4 = round(self.v4, read_u32(&block[12..]));
    }
}

fn round(acc: u32, input: u32) -> u32 {
    acc.wrapping_add(input.wrapping_mul(0x85EB_CA77))
        .rotate_left(13)
        .wrapping_mul(0x9E37_79B1)
}

fn read_u32(input: &[u8]) -> u32 {
    u32::from_le_bytes([input[0], input[1], input[2], input[3]])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_round_trip_repeated() {
        let input = b"Some data Some data Some data Some data";
        let mut compressed = vec![0u8; unsafe { LZ4_compressBound(input.len() as c_int) } as usize];
        let clen = unsafe {
            LZ4_compress_default(
                input.as_ptr() as *const c_char,
                compressed.as_mut_ptr() as *mut c_char,
                input.len() as c_int,
                compressed.len() as c_int,
            )
        };
        assert!(clen > 0);
        let mut output = vec![0u8; input.len()];
        let olen = unsafe {
            LZ4_decompress_safe(
                compressed.as_ptr() as *const c_char,
                output.as_mut_ptr() as *mut c_char,
                clen,
                output.len() as c_int,
            )
        };
        assert_eq!(olen as usize, input.len());
        assert_eq!(&output, input);
    }

    #[test]
    fn block_dest_size_and_ext_state_round_trip() {
        let input = b"dest-size-data-".repeat(2048);
        let bound = unsafe { LZ4_compressBound(input.len() as c_int) } as usize;
        let mut state = vec![0u8; LZ4_sizeofState() as usize];
        let mut compressed = vec![0u8; bound];
        let compressed_len = unsafe {
            LZ4_compress_fast_extState(
                state.as_mut_ptr() as *mut c_void,
                input.as_ptr() as *const c_char,
                compressed.as_mut_ptr() as *mut c_char,
                input.len() as c_int,
                compressed.len() as c_int,
                1,
            )
        };
        assert!(compressed_len > 0);

        let mut output = vec![0u8; input.len()];
        let output_len = unsafe {
            LZ4_decompress_safe(
                compressed.as_ptr() as *const c_char,
                output.as_mut_ptr() as *mut c_char,
                compressed_len,
                output.len() as c_int,
            )
        };
        assert_eq!(output_len as usize, input.len());
        assert_eq!(output, input);

        let mut src_size = input.len() as c_int;
        let mut tiny = vec![0u8; compressed_len as usize / 2];
        let tiny_len = unsafe {
            LZ4_compress_destSize(
                input.as_ptr() as *const c_char,
                tiny.as_mut_ptr() as *mut c_char,
                &mut src_size,
                tiny.len() as c_int,
            )
        };
        assert!(tiny_len > 0);
        assert!(src_size > 0);
        assert!(src_size < input.len() as c_int);
    }

    #[test]
    fn fast_stream_compression_references_loaded_dictionary() {
        unsafe {
            let dict = b"abcdefghijklmnop";
            let input = b"abcdefghijklmnop";
            let stream = LZ4_createStream();
            assert!(!stream.is_null());
            assert_eq!(
                LZ4_loadDict(stream, dict.as_ptr() as *const c_char, dict.len() as c_int),
                dict.len() as c_int
            );
            let mut compressed = vec![0u8; LZ4_compressBound(input.len() as c_int) as usize];
            let compressed_len = LZ4_compress_continue(
                stream,
                input.as_ptr() as *const c_char,
                compressed.as_mut_ptr() as *mut c_char,
                input.len() as c_int,
                compressed.len() as c_int,
            );
            assert!(compressed_len > 0);
            assert!((compressed_len as usize) < input.len() + 1);

            let mut saved = vec![0u8; dict.len() + input.len()];
            let saved_len = LZ4_saveDict(
                stream,
                saved.as_mut_ptr() as *mut c_char,
                saved.len() as c_int,
            );
            assert!(saved_len >= input.len() as c_int);
            LZ4_freeStream(stream);

            let mut output = vec![0u8; input.len()];
            let output_len = LZ4_decompress_safe_usingDict(
                compressed.as_ptr() as *const c_char,
                output.as_mut_ptr() as *mut c_char,
                compressed_len,
                output.len() as c_int,
                dict.as_ptr() as *const c_char,
                dict.len() as c_int,
            );
            assert_eq!(output_len as usize, input.len());
            assert_eq!(output, input);
        }
    }

    #[test]
    fn decompress_safe_partial_returns_prefix() {
        let input = b"partial output data ".repeat(1024);
        let bound = unsafe { LZ4_compressBound(input.len() as c_int) } as usize;
        let mut compressed = vec![0u8; bound];
        let compressed_len = unsafe {
            LZ4_compress_default(
                input.as_ptr() as *const c_char,
                compressed.as_mut_ptr() as *mut c_char,
                input.len() as c_int,
                compressed.len() as c_int,
            )
        };
        assert!(compressed_len > 0);
        let target = 1234usize;
        let mut output = vec![0u8; input.len()];
        let output_len = unsafe {
            LZ4_decompress_safe_partial(
                compressed.as_ptr() as *const c_char,
                output.as_mut_ptr() as *mut c_char,
                compressed_len,
                target as c_int,
                output.len() as c_int,
            )
        };
        assert_eq!(output_len as usize, target);
        assert_eq!(&output[..target], &input[..target]);
    }

    #[test]
    fn decompress_fast_returns_consumed_bytes() {
        let input = b"fast decode consumed bytes ".repeat(256);
        let bound = unsafe { LZ4_compressBound(input.len() as c_int) } as usize;
        let mut compressed = vec![0u8; bound];
        let compressed_len = unsafe {
            LZ4_compress_default(
                input.as_ptr() as *const c_char,
                compressed.as_mut_ptr() as *mut c_char,
                input.len() as c_int,
                compressed.len() as c_int,
            )
        };
        assert!(compressed_len > 0);
        let mut output = vec![0u8; input.len()];
        let consumed = unsafe {
            LZ4_decompress_fast(
                compressed.as_ptr() as *const c_char,
                output.as_mut_ptr() as *mut c_char,
                output.len() as c_int,
            )
        };
        assert!(consumed > 0);
        assert!(consumed <= compressed_len);
        assert_eq!(output, input);

        let dict = b"abc";
        let dict_compressed = [0x30, b'x', b'y', b'z'];
        let mut dict_output = vec![0u8; 3];
        let consumed = unsafe {
            LZ4_decompress_fast_usingDict(
                dict_compressed.as_ptr() as *const c_char,
                dict_output.as_mut_ptr() as *mut c_char,
                dict_output.len() as c_int,
                dict.as_ptr() as *const c_char,
                dict.len() as c_int,
            )
        };
        assert_eq!(consumed, dict_compressed.len() as c_int);
        assert_eq!(dict_output, b"xyz");
    }

    #[test]
    fn decodes_known_empty_block() {
        let compressed = [0u8];
        let mut output = [];
        let len = unsafe {
            LZ4_decompress_safe(
                compressed.as_ptr() as *const c_char,
                output.as_mut_ptr() as *mut c_char,
                compressed.len() as c_int,
                0,
            )
        };
        assert_eq!(len, 0);
    }

    #[test]
    fn streaming_decode_uses_dictionary() {
        unsafe {
            let compressed = [0x02, 0x03, 0x00];
            let mut direct = vec![0u8; 6];
            assert_eq!(
                decompress_block_with_dict(&compressed, &mut direct, b"abc"),
                Some(6)
            );
            let dict = b"abc";
            let mut using_dict = vec![0u8; 6];
            let using_dict_len = LZ4_decompress_safe_usingDict(
                compressed.as_ptr() as *const c_char,
                using_dict.as_mut_ptr() as *mut c_char,
                compressed.len() as c_int,
                using_dict.len() as c_int,
                dict.as_ptr() as *const c_char,
                dict.len() as c_int,
            );
            assert_eq!(using_dict_len, using_dict.len() as c_int);
            assert_eq!(using_dict, b"abcabc");

            let stream = LZ4_createStreamDecode();
            assert!(!stream.is_null());
            assert_eq!(
                LZ4_setStreamDecode(stream, dict.as_ptr() as *const c_char, dict.len() as c_int),
                1
            );
            let mut output = vec![0u8; 6];
            let len = LZ4_decompress_safe_continue(
                stream,
                compressed.as_ptr() as *const c_char,
                output.as_mut_ptr() as *mut c_char,
                compressed.len() as c_int,
                output.len() as c_int,
            );
            assert_eq!(len, output.len() as c_int);
            assert_eq!(output, b"abcabc");
            LZ4_freeStreamDecode(stream);
        }
    }

    #[test]
    fn decompress_match_copy_handles_small_overlap() {
        let compressed = [0x15, b'a', 0x01, 0x00];
        let mut output = vec![0u8; 10];

        assert_eq!(decompress_block(&compressed, &mut output), Some(10));
        assert_eq!(output, b"aaaaaaaaaa");

        let mut partial = vec![0u8; 10];
        assert_eq!(
            decompress_block_partial(&compressed, &mut partial, 7),
            Some(7)
        );
        assert_eq!(&partial[..7], b"aaaaaaa");
    }

    #[test]
    fn decompress_match_copy_spans_dictionary_and_prefix() {
        let compressed = [0x24, b'd', b'e', 0x04, 0x00];
        let mut output = vec![0u8; 10];

        assert_eq!(
            decompress_block_with_dict(&compressed, &mut output, b"abc"),
            Some(10)
        );
        assert_eq!(output, b"debcdebcde");

        let mut fast_output = vec![0u8; 10];
        let consumed = unsafe {
            LZ4_decompress_fast_usingDict(
                compressed.as_ptr() as *const c_char,
                fast_output.as_mut_ptr() as *mut c_char,
                fast_output.len() as c_int,
                b"abc".as_ptr() as *const c_char,
                3,
            )
        };
        assert_eq!(consumed, compressed.len() as c_int);
        assert_eq!(fast_output, b"debcdebcde");
    }

    #[test]
    fn frame_round_trip_raw_blocks() {
        unsafe {
            let mut cctx = LZ4FCompressionContext(ptr::null_mut());
            assert_eq!(LZ4F_createCompressionContext(&mut cctx, LZ4F_VERSION), 0);
            let prefs = LZ4FPreferences {
                frame_info: LZ4FFrameInfo {
                    block_size_id: BlockSize::Max64KB,
                    block_mode: BlockMode::Independent,
                    content_checksum_flag: ContentChecksum::ChecksumEnabled,
                    frame_type: FrameType::Frame,
                    content_size: 0,
                    dict_id: 0,
                    block_checksum_flag: BlockChecksum::BlockChecksumEnabled,
                },
                compression_level: 0,
                auto_flush: 0,
                favor_dec_speed: 0,
                reserved: [0; 3],
            };
            let input = b"frame data";
            let mut encoded = vec![0u8; 128];
            let mut pos = LZ4F_compressBegin(cctx, encoded.as_mut_ptr(), encoded.len(), &prefs);
            pos += LZ4F_compressUpdate(
                cctx,
                encoded.as_mut_ptr().add(pos),
                encoded.len() - pos,
                input.as_ptr(),
                input.len(),
                ptr::null(),
            );
            pos += LZ4F_compressEnd(
                cctx,
                encoded.as_mut_ptr().add(pos),
                encoded.len() - pos,
                ptr::null(),
            );
            encoded.truncate(pos);
            LZ4F_freeCompressionContext(cctx);

            let mut dctx = LZ4FDecompressionContext(ptr::null_mut());
            assert_eq!(LZ4F_createDecompressionContext(&mut dctx, LZ4F_VERSION), 0);
            let mut out = vec![0u8; input.len()];
            let mut dst_size = out.len();
            let mut src_size = encoded.len();
            let code = LZ4F_decompress(
                dctx,
                out.as_mut_ptr(),
                &mut dst_size,
                encoded.as_ptr(),
                &mut src_size,
                ptr::null(),
            );
            assert!(!LZ4F_isError(code).eq(&1));
            assert_eq!(dst_size, input.len());
            assert_eq!(&out, input);
            LZ4F_freeDecompressionContext(dctx);
        }
    }

    #[test]
    fn frame_single_call_compress_round_trip() {
        unsafe {
            assert_eq!(LZ4F_getVersion(), LZ4F_VERSION);
            assert_eq!(LZ4F_compressionLevel_max(), LZ4HC_CLEVEL_MAX);
            let prefs = LZ4FPreferences {
                frame_info: LZ4FFrameInfo {
                    block_size_id: BlockSize::Max64KB,
                    block_mode: BlockMode::Linked,
                    content_checksum_flag: ContentChecksum::ChecksumEnabled,
                    frame_type: FrameType::Frame,
                    content_size: 0,
                    dict_id: 0,
                    block_checksum_flag: BlockChecksum::BlockChecksumEnabled,
                },
                compression_level: 9,
                auto_flush: 0,
                favor_dec_speed: 0,
                reserved: [0; 3],
            };
            let input = b"single call frame compression ".repeat(512);
            let bound = LZ4F_compressFrameBound(input.len(), &prefs);
            let mut encoded = vec![0u8; bound];
            let encoded_len = LZ4F_compressFrame(
                encoded.as_mut_ptr() as *mut c_void,
                encoded.len(),
                input.as_ptr() as *const c_void,
                input.len(),
                &prefs,
            );
            assert_eq!(LZ4F_isError(encoded_len), 0);
            encoded.truncate(encoded_len);

            let mut dctx = LZ4FDecompressionContext(ptr::null_mut());
            assert_eq!(LZ4F_createDecompressionContext(&mut dctx, LZ4F_VERSION), 0);
            let mut output = vec![0u8; input.len()];
            let mut src_offset = 0usize;
            let mut dst_offset = 0usize;
            loop {
                let mut src_size = encoded.len() - src_offset;
                let mut dst_size = output.len() - dst_offset;
                let code = LZ4F_decompress(
                    dctx,
                    output[dst_offset..].as_mut_ptr(),
                    &mut dst_size,
                    encoded.as_ptr().add(src_offset),
                    &mut src_size,
                    ptr::null(),
                );
                assert_eq!(LZ4F_isError(code), 0);
                src_offset += src_size;
                dst_offset += dst_size;
                if code == 0 {
                    break;
                }
            }
            assert_eq!(dst_offset, output.len());
            assert_eq!(output, input);
            LZ4F_freeDecompressionContext(dctx);
        }
    }

    #[test]
    fn frame_header_size_reports_expected_lengths() {
        unsafe {
            let mut cctx = LZ4FCompressionContext(ptr::null_mut());
            assert_eq!(LZ4F_createCompressionContext(&mut cctx, LZ4F_VERSION), 0);
            let prefs = LZ4FPreferences {
                frame_info: LZ4FFrameInfo {
                    block_size_id: BlockSize::Max64KB,
                    block_mode: BlockMode::Independent,
                    content_checksum_flag: ContentChecksum::ChecksumEnabled,
                    frame_type: FrameType::Frame,
                    content_size: 123,
                    dict_id: 0,
                    block_checksum_flag: BlockChecksum::NoBlockChecksum,
                },
                compression_level: 0,
                auto_flush: 0,
                favor_dec_speed: 0,
                reserved: [0; 3],
            };
            let mut encoded = vec![0u8; 32];
            let header_len = LZ4F_compressBegin(cctx, encoded.as_mut_ptr(), encoded.len(), &prefs);
            assert_eq!(
                LZ4F_headerSize(encoded.as_ptr() as *const c_void, 6),
                header_len
            );
            assert_eq!(
                LZ4F_headerSize(encoded.as_ptr() as *const c_void, 5),
                ERROR_BAD_HEADER
            );
            LZ4F_freeCompressionContext(cctx);

            let mut skippable = Vec::new();
            skippable.extend_from_slice(&LZ4F_SKIPPABLE_MAGIC_MIN.to_le_bytes());
            skippable.extend_from_slice(&0u32.to_le_bytes());
            assert_eq!(
                LZ4F_headerSize(skippable.as_ptr() as *const c_void, skippable.len()),
                8
            );
        }
    }

    #[test]
    fn frame_using_dict_round_trip_first_independent_block() {
        unsafe {
            let dict = b"abcdefghijklmnop";
            let input = b"abcdefghijklmnop";
            let prefs = LZ4FPreferences {
                frame_info: LZ4FFrameInfo {
                    block_size_id: BlockSize::Max64KB,
                    block_mode: BlockMode::Independent,
                    content_checksum_flag: ContentChecksum::NoChecksum,
                    frame_type: FrameType::Frame,
                    content_size: input.len() as u64,
                    dict_id: 0,
                    block_checksum_flag: BlockChecksum::NoBlockChecksum,
                },
                compression_level: 9,
                auto_flush: 0,
                favor_dec_speed: 0,
                reserved: [0; 3],
            };

            let mut cctx = LZ4FCompressionContext(ptr::null_mut());
            assert_eq!(LZ4F_createCompressionContext(&mut cctx, LZ4F_VERSION), 0);
            let mut encoded = vec![0u8; 256];
            let mut pos = LZ4F_compressBegin_usingDict(
                cctx,
                encoded.as_mut_ptr() as *mut c_void,
                encoded.len(),
                dict.as_ptr() as *const c_void,
                dict.len(),
                &prefs,
            );
            assert_eq!(LZ4F_isError(pos), 0);
            let update_len = LZ4F_compressUpdate(
                cctx,
                encoded.as_mut_ptr().add(pos),
                encoded.len() - pos,
                input.as_ptr(),
                input.len(),
                ptr::null(),
            );
            assert_eq!(LZ4F_isError(update_len), 0);
            let block_header = u32::from_le_bytes(encoded[pos..pos + 4].try_into().unwrap());
            assert_eq!(block_header & 0x8000_0000, 0);
            assert!((block_header as usize) < input.len());
            pos += update_len;
            pos += LZ4F_compressEnd(
                cctx,
                encoded.as_mut_ptr().add(pos),
                encoded.len() - pos,
                ptr::null(),
            );
            encoded.truncate(pos);
            LZ4F_freeCompressionContext(cctx);

            let mut dctx = LZ4FDecompressionContext(ptr::null_mut());
            assert_eq!(LZ4F_createDecompressionContext(&mut dctx, LZ4F_VERSION), 0);
            let mut output = vec![0u8; input.len()];
            let mut dst_size = output.len();
            let mut src_size = encoded.len();
            let code = LZ4F_decompress_usingDict(
                dctx,
                output.as_mut_ptr() as *mut c_void,
                &mut dst_size,
                encoded.as_ptr() as *const c_void,
                &mut src_size,
                dict.as_ptr() as *const c_void,
                dict.len(),
                ptr::null(),
            );
            assert_eq!(LZ4F_isError(code), 0);
            assert_eq!(dst_size, input.len());
            assert_eq!(output, input);
            LZ4F_freeDecompressionContext(dctx);
        }
    }

    #[test]
    fn frame_cdict_round_trip() {
        unsafe {
            let dict = b"abcdefghijklmnop";
            let input = b"abcdefghijklmnop";
            let cdict = LZ4F_createCDict(dict.as_ptr() as *const c_void, dict.len());
            assert!(!cdict.is_null());
            let prefs = LZ4FPreferences {
                frame_info: LZ4FFrameInfo {
                    block_size_id: BlockSize::Max64KB,
                    block_mode: BlockMode::Independent,
                    content_checksum_flag: ContentChecksum::NoChecksum,
                    frame_type: FrameType::Frame,
                    content_size: input.len() as u64,
                    dict_id: 0,
                    block_checksum_flag: BlockChecksum::NoBlockChecksum,
                },
                compression_level: 9,
                auto_flush: 0,
                favor_dec_speed: 0,
                reserved: [0; 3],
            };

            let mut cctx = LZ4FCompressionContext(ptr::null_mut());
            assert_eq!(LZ4F_createCompressionContext(&mut cctx, LZ4F_VERSION), 0);
            let mut encoded = vec![0u8; 256];
            let encoded_len = LZ4F_compressFrame_usingCDict(
                cctx,
                encoded.as_mut_ptr() as *mut c_void,
                encoded.len(),
                input.as_ptr() as *const c_void,
                input.len(),
                cdict,
                &prefs,
            );
            assert_eq!(LZ4F_isError(encoded_len), 0);
            encoded.truncate(encoded_len);
            LZ4F_freeCompressionContext(cctx);
            LZ4F_freeCDict(cdict);

            let mut dctx = LZ4FDecompressionContext(ptr::null_mut());
            assert_eq!(LZ4F_createDecompressionContext(&mut dctx, LZ4F_VERSION), 0);
            let mut output = vec![0u8; input.len()];
            let mut dst_size = output.len();
            let mut src_size = encoded.len();
            let code = LZ4F_decompress_usingDict(
                dctx,
                output.as_mut_ptr() as *mut c_void,
                &mut dst_size,
                encoded.as_ptr() as *const c_void,
                &mut src_size,
                dict.as_ptr() as *const c_void,
                dict.len(),
                ptr::null(),
            );
            assert_eq!(LZ4F_isError(code), 0);
            assert_eq!(dst_size, input.len());
            assert_eq!(output, input);
            LZ4F_freeDecompressionContext(dctx);
        }
    }

    #[test]
    fn frame_skippable_frame_is_skipped() {
        unsafe {
            let mut skippable = Vec::new();
            skippable.extend_from_slice(&LZ4F_SKIPPABLE_MAGIC_MIN.to_le_bytes());
            skippable.extend_from_slice(&5u32.to_le_bytes());
            skippable.extend_from_slice(b"abcde");

            let mut dctx_info = LZ4FDecompressionContext(ptr::null_mut());
            assert_eq!(
                LZ4F_createDecompressionContext(&mut dctx_info, LZ4F_VERSION),
                0
            );
            let mut info = LZ4FFrameInfo {
                block_size_id: BlockSize::Default,
                block_mode: BlockMode::Independent,
                content_checksum_flag: ContentChecksum::NoChecksum,
                frame_type: FrameType::Frame,
                content_size: 0,
                dict_id: 0,
                block_checksum_flag: BlockChecksum::NoBlockChecksum,
            };
            let mut info_src_size = skippable.len();
            assert_eq!(
                LZ4F_getFrameInfo(dctx_info, &mut info, skippable.as_ptr(), &mut info_src_size),
                0
            );
            assert!(matches!(info.frame_type, FrameType::SkippableFrame));
            assert_eq!(info_src_size, skippable.len());
            LZ4F_freeDecompressionContext(dctx_info);

            let mut dctx = LZ4FDecompressionContext(ptr::null_mut());
            assert_eq!(LZ4F_createDecompressionContext(&mut dctx, LZ4F_VERSION), 0);
            let mut output = [0u8; 1];
            let mut dst_size = output.len();
            let mut src_size = skippable.len();
            let code = LZ4F_decompress(
                dctx,
                output.as_mut_ptr(),
                &mut dst_size,
                skippable.as_ptr(),
                &mut src_size,
                ptr::null(),
            );
            assert_eq!(code, 0);
            assert_eq!(dst_size, 0);
            assert_eq!(src_size, skippable.len());
            LZ4F_freeDecompressionContext(dctx);
        }
    }

    #[test]
    fn frame_update_compresses_blocks_when_level_is_set() {
        unsafe {
            let mut cctx = LZ4FCompressionContext(ptr::null_mut());
            assert_eq!(LZ4F_createCompressionContext(&mut cctx, LZ4F_VERSION), 0);
            let prefs = LZ4FPreferences {
                frame_info: LZ4FFrameInfo {
                    block_size_id: BlockSize::Max64KB,
                    block_mode: BlockMode::Independent,
                    content_checksum_flag: ContentChecksum::NoChecksum,
                    frame_type: FrameType::Frame,
                    content_size: 0,
                    dict_id: 0,
                    block_checksum_flag: BlockChecksum::NoBlockChecksum,
                },
                compression_level: 9,
                auto_flush: 0,
                favor_dec_speed: 0,
                reserved: [0; 3],
            };
            let input = vec![b'a'; 16 * 1024];
            let mut encoded = vec![0u8; LZ4F_compressBound(input.len(), &prefs) + 32];
            let header_len = LZ4F_compressBegin(cctx, encoded.as_mut_ptr(), encoded.len(), &prefs);
            assert!(!LZ4F_isError(header_len).eq(&1));
            let update_len = LZ4F_compressUpdate(
                cctx,
                encoded.as_mut_ptr().add(header_len),
                encoded.len() - header_len,
                input.as_ptr(),
                input.len(),
                ptr::null(),
            );
            assert!(!LZ4F_isError(update_len).eq(&1));
            let block_header =
                u32::from_le_bytes(encoded[header_len..header_len + 4].try_into().unwrap());
            assert_eq!(block_header & 0x8000_0000, 0);
            assert!((block_header as usize) < input.len());

            let end_len = LZ4F_compressEnd(
                cctx,
                encoded.as_mut_ptr().add(header_len + update_len),
                encoded.len() - header_len - update_len,
                ptr::null(),
            );
            assert!(!LZ4F_isError(end_len).eq(&1));
            encoded.truncate(header_len + update_len + end_len);
            LZ4F_freeCompressionContext(cctx);

            let mut dctx = LZ4FDecompressionContext(ptr::null_mut());
            assert_eq!(LZ4F_createDecompressionContext(&mut dctx, LZ4F_VERSION), 0);
            let mut output = vec![0u8; input.len()];
            let mut src_size = encoded.len();
            let mut dst_size = output.len();
            let code = LZ4F_decompress(
                dctx,
                output.as_mut_ptr(),
                &mut dst_size,
                encoded.as_ptr(),
                &mut src_size,
                ptr::null(),
            );
            assert!(!LZ4F_isError(code).eq(&1));
            assert_eq!(dst_size, input.len());
            assert_eq!(output, input);
            LZ4F_freeDecompressionContext(dctx);
        }
    }

    #[test]
    fn frame_linked_blocks_use_previous_block_history() {
        unsafe {
            let mut cctx = LZ4FCompressionContext(ptr::null_mut());
            assert_eq!(LZ4F_createCompressionContext(&mut cctx, LZ4F_VERSION), 0);
            let prefs = LZ4FPreferences {
                frame_info: LZ4FFrameInfo {
                    block_size_id: BlockSize::Max64KB,
                    block_mode: BlockMode::Linked,
                    content_checksum_flag: ContentChecksum::NoChecksum,
                    frame_type: FrameType::Frame,
                    content_size: 0,
                    dict_id: 0,
                    block_checksum_flag: BlockChecksum::NoBlockChecksum,
                },
                compression_level: 9,
                auto_flush: 0,
                favor_dec_speed: 0,
                reserved: [0; 3],
            };
            let first = b"abcdefghijklmnop";
            let second = b"abcdefghijklmnop";
            let mut encoded = vec![0u8; 256];
            let mut pos = LZ4F_compressBegin(cctx, encoded.as_mut_ptr(), encoded.len(), &prefs);

            let mut dctx_info = LZ4FDecompressionContext(ptr::null_mut());
            assert_eq!(
                LZ4F_createDecompressionContext(&mut dctx_info, LZ4F_VERSION),
                0
            );
            let mut info = LZ4FFrameInfo {
                block_size_id: BlockSize::Default,
                block_mode: BlockMode::Independent,
                content_checksum_flag: ContentChecksum::NoChecksum,
                frame_type: FrameType::Frame,
                content_size: 0,
                dict_id: 0,
                block_checksum_flag: BlockChecksum::NoBlockChecksum,
            };
            let mut header_size = pos;
            assert_eq!(
                LZ4F_getFrameInfo(dctx_info, &mut info, encoded.as_ptr(), &mut header_size),
                0
            );
            assert!(matches!(info.block_mode, BlockMode::Linked));
            LZ4F_freeDecompressionContext(dctx_info);

            pos += LZ4F_compressUpdate(
                cctx,
                encoded.as_mut_ptr().add(pos),
                encoded.len() - pos,
                first.as_ptr(),
                first.len(),
                ptr::null(),
            );
            let second_block_at = pos;
            let second_len = LZ4F_compressUpdate(
                cctx,
                encoded.as_mut_ptr().add(pos),
                encoded.len() - pos,
                second.as_ptr(),
                second.len(),
                ptr::null(),
            );
            pos += second_len;
            let second_header = u32::from_le_bytes(
                encoded[second_block_at..second_block_at + 4]
                    .try_into()
                    .unwrap(),
            );
            assert_eq!(second_header & 0x8000_0000, 0);
            assert!((second_header as usize) < second.len() + 1);
            pos += LZ4F_compressEnd(
                cctx,
                encoded.as_mut_ptr().add(pos),
                encoded.len() - pos,
                ptr::null(),
            );
            encoded.truncate(pos);
            LZ4F_freeCompressionContext(cctx);

            let mut dctx = LZ4FDecompressionContext(ptr::null_mut());
            assert_eq!(LZ4F_createDecompressionContext(&mut dctx, LZ4F_VERSION), 0);
            let mut output = vec![0u8; first.len() + second.len()];
            let mut src_offset = 0usize;
            let mut dst_offset = 0usize;
            loop {
                let mut src_size = encoded.len() - src_offset;
                let mut dst_size = output.len() - dst_offset;
                let code = LZ4F_decompress(
                    dctx,
                    output[dst_offset..].as_mut_ptr(),
                    &mut dst_size,
                    encoded.as_ptr().add(src_offset),
                    &mut src_size,
                    ptr::null(),
                );
                assert_eq!(LZ4F_isError(code), 0);
                src_offset += src_size;
                dst_offset += dst_size;
                if code == 0 {
                    break;
                }
            }
            assert_eq!(dst_offset, output.len());
            assert_eq!(&output[..first.len()], first);
            assert_eq!(&output[first.len()..], second);
            LZ4F_freeDecompressionContext(dctx);
        }
    }

    #[test]
    fn frame_decompress_rejects_bad_checksums() {
        unsafe {
            let mut cctx = LZ4FCompressionContext(ptr::null_mut());
            assert_eq!(LZ4F_createCompressionContext(&mut cctx, LZ4F_VERSION), 0);
            let prefs = LZ4FPreferences {
                frame_info: LZ4FFrameInfo {
                    block_size_id: BlockSize::Max64KB,
                    block_mode: BlockMode::Independent,
                    content_checksum_flag: ContentChecksum::ChecksumEnabled,
                    frame_type: FrameType::Frame,
                    content_size: 0,
                    dict_id: 0,
                    block_checksum_flag: BlockChecksum::BlockChecksumEnabled,
                },
                compression_level: 9,
                auto_flush: 0,
                favor_dec_speed: 0,
                reserved: [0; 3],
            };
            let input = b"checksum data ".repeat(1024);
            let mut encoded = vec![0u8; LZ4F_compressBound(input.len(), &prefs) + 32];
            let header_len = LZ4F_compressBegin(cctx, encoded.as_mut_ptr(), encoded.len(), &prefs);
            let update_len = LZ4F_compressUpdate(
                cctx,
                encoded.as_mut_ptr().add(header_len),
                encoded.len() - header_len,
                input.as_ptr(),
                input.len(),
                ptr::null(),
            );
            let end_len = LZ4F_compressEnd(
                cctx,
                encoded.as_mut_ptr().add(header_len + update_len),
                encoded.len() - header_len - update_len,
                ptr::null(),
            );
            encoded.truncate(header_len + update_len + end_len);
            LZ4F_freeCompressionContext(cctx);

            let mut bad_block = encoded.clone();
            let block_len =
                (u32::from_le_bytes(bad_block[header_len..header_len + 4].try_into().unwrap())
                    & 0x7FFF_FFFF) as usize;
            bad_block[header_len + 4 + block_len] ^= 0x80;
            assert_corrupt_frame_fails("block checksum", &bad_block, input.len());

            let mut bad_content = encoded;
            let last = bad_content.len() - 1;
            bad_content[last] ^= 0x80;
            assert_corrupt_frame_fails("content checksum", &bad_content, input.len());
        }
    }

    #[test]
    fn frame_decompress_rejects_bad_header_checksum() {
        unsafe {
            let mut cctx = LZ4FCompressionContext(ptr::null_mut());
            assert_eq!(LZ4F_createCompressionContext(&mut cctx, LZ4F_VERSION), 0);
            let prefs = LZ4FPreferences {
                frame_info: LZ4FFrameInfo {
                    block_size_id: BlockSize::Max64KB,
                    block_mode: BlockMode::Independent,
                    content_checksum_flag: ContentChecksum::NoChecksum,
                    frame_type: FrameType::Frame,
                    content_size: 0,
                    dict_id: 0,
                    block_checksum_flag: BlockChecksum::NoBlockChecksum,
                },
                compression_level: 0,
                auto_flush: 0,
                favor_dec_speed: 0,
                reserved: [0; 3],
            };
            let input = b"header checksum";
            let mut encoded = vec![0u8; 128];
            let mut pos = LZ4F_compressBegin(cctx, encoded.as_mut_ptr(), encoded.len(), &prefs);
            encoded[pos - 1] ^= 0x80;
            pos += LZ4F_compressUpdate(
                cctx,
                encoded.as_mut_ptr().add(pos),
                encoded.len() - pos,
                input.as_ptr(),
                input.len(),
                ptr::null(),
            );
            pos += LZ4F_compressEnd(
                cctx,
                encoded.as_mut_ptr().add(pos),
                encoded.len() - pos,
                ptr::null(),
            );
            encoded.truncate(pos);
            LZ4F_freeCompressionContext(cctx);

            let mut dctx = LZ4FDecompressionContext(ptr::null_mut());
            assert_eq!(LZ4F_createDecompressionContext(&mut dctx, LZ4F_VERSION), 0);
            let mut output = vec![0u8; input.len()];
            let mut src_size = encoded.len();
            let mut dst_size = output.len();
            let code = LZ4F_decompress(
                dctx,
                output.as_mut_ptr(),
                &mut dst_size,
                encoded.as_ptr(),
                &mut src_size,
                ptr::null(),
            );
            assert_eq!(code, ERROR_BAD_HEADER);
            LZ4F_freeDecompressionContext(dctx);
        }
    }

    #[test]
    fn frame_decompress_rejects_content_size_mismatch() {
        unsafe {
            let mut cctx = LZ4FCompressionContext(ptr::null_mut());
            assert_eq!(LZ4F_createCompressionContext(&mut cctx, LZ4F_VERSION), 0);
            let input = b"wrong content size";
            let prefs = LZ4FPreferences {
                frame_info: LZ4FFrameInfo {
                    block_size_id: BlockSize::Max64KB,
                    block_mode: BlockMode::Independent,
                    content_checksum_flag: ContentChecksum::NoChecksum,
                    frame_type: FrameType::Frame,
                    content_size: input.len() as u64 + 1,
                    dict_id: 0,
                    block_checksum_flag: BlockChecksum::NoBlockChecksum,
                },
                compression_level: 0,
                auto_flush: 0,
                favor_dec_speed: 0,
                reserved: [0; 3],
            };
            let mut encoded = vec![0u8; 128];
            let mut pos = LZ4F_compressBegin(cctx, encoded.as_mut_ptr(), encoded.len(), &prefs);
            pos += LZ4F_compressUpdate(
                cctx,
                encoded.as_mut_ptr().add(pos),
                encoded.len() - pos,
                input.as_ptr(),
                input.len(),
                ptr::null(),
            );
            pos += LZ4F_compressEnd(
                cctx,
                encoded.as_mut_ptr().add(pos),
                encoded.len() - pos,
                ptr::null(),
            );
            encoded.truncate(pos);
            LZ4F_freeCompressionContext(cctx);

            let mut dctx = LZ4FDecompressionContext(ptr::null_mut());
            assert_eq!(LZ4F_createDecompressionContext(&mut dctx, LZ4F_VERSION), 0);
            let mut output = vec![0u8; input.len()];
            let mut src_size = encoded.len();
            let mut dst_size = output.len();
            let code = LZ4F_decompress(
                dctx,
                output.as_mut_ptr(),
                &mut dst_size,
                encoded.as_ptr(),
                &mut src_size,
                ptr::null(),
            );
            assert_eq!(code, ERROR_GENERIC);
            LZ4F_freeDecompressionContext(dctx);
        }
    }

    fn assert_corrupt_frame_fails(kind: &str, encoded: &[u8], output_len: usize) {
        unsafe {
            let mut dctx = LZ4FDecompressionContext(ptr::null_mut());
            assert_eq!(LZ4F_createDecompressionContext(&mut dctx, LZ4F_VERSION), 0);
            let mut output = vec![0u8; output_len];
            let mut offset = 0usize;
            for _ in 0..8 {
                let mut src_size = encoded.len() - offset;
                let src_ptr = if src_size == 0 {
                    ptr::null()
                } else {
                    encoded.as_ptr().add(offset)
                };
                let mut dst_size = output.len();
                let code = LZ4F_decompress(
                    dctx,
                    output.as_mut_ptr(),
                    &mut dst_size,
                    src_ptr,
                    &mut src_size,
                    ptr::null(),
                );
                if code == ERROR_CHECKSUM_INVALID {
                    LZ4F_freeDecompressionContext(dctx);
                    return;
                }
                assert_eq!(LZ4F_isError(code), 0);
                assert_ne!(code, 0, "{kind} corrupt frame decoded successfully");
                offset += src_size;
            }
            panic!("{kind} corrupt frame did not report checksum failure");
        }
    }

    #[test]
    fn hc_round_trip_and_improves_repetitive_block() {
        let mut input = Vec::new();
        for _ in 0..4096 {
            input.extend_from_slice(b"the quick brown fox jumps over the lazy dog. ");
        }

        let bound = unsafe { LZ4_compressBound(input.len() as c_int) } as usize;
        let mut fast = vec![0u8; bound];
        let fast_len = unsafe {
            LZ4_compress_default(
                input.as_ptr() as *const c_char,
                fast.as_mut_ptr() as *mut c_char,
                input.len() as c_int,
                fast.len() as c_int,
            )
        };
        assert!(fast_len > 0);

        let mut hc = vec![0u8; bound];
        let hc_len = unsafe {
            LZ4_compress_HC(
                input.as_ptr() as *const c_char,
                hc.as_mut_ptr() as *mut c_char,
                input.len() as c_int,
                hc.len() as c_int,
                9,
            )
        };
        assert!(hc_len > 0);
        assert!(hc_len <= fast_len);

        let mut output = vec![0u8; input.len()];
        let output_len = unsafe {
            LZ4_decompress_safe(
                hc.as_ptr() as *const c_char,
                output.as_mut_ptr() as *mut c_char,
                hc_len,
                output.len() as c_int,
            )
        };
        assert_eq!(output_len as usize, input.len());
        assert_eq!(output, input);
    }

    #[test]
    fn hc_levels_round_trip_varied_inputs() {
        let mut inputs = Vec::new();
        inputs.push((0..4096).map(|n| (n & 0xff) as u8).collect::<Vec<_>>());
        inputs.push(vec![b'a'; 128 * 1024]);
        let mut patterned = Vec::new();
        for n in 0..32 * 1024 {
            patterned.push(if n % 97 < 64 {
                b"ACGT"[(n / 3) % 4]
            } else {
                (n & 0xff) as u8
            });
        }
        inputs.push(patterned);

        for input in inputs {
            let bound = unsafe { LZ4_compressBound(input.len() as c_int) } as usize;
            for level in 1..=12 {
                let mut compressed = vec![0u8; bound];
                let compressed_len = unsafe {
                    LZ4_compress_HC(
                        input.as_ptr() as *const c_char,
                        compressed.as_mut_ptr() as *mut c_char,
                        input.len() as c_int,
                        compressed.len() as c_int,
                        level,
                    )
                };
                assert!(compressed_len > 0, "level {level}");

                let mut output = vec![0u8; input.len()];
                let output_len = unsafe {
                    LZ4_decompress_safe(
                        compressed.as_ptr() as *const c_char,
                        output.as_mut_ptr() as *mut c_char,
                        compressed_len,
                        output.len() as c_int,
                    )
                };
                assert_eq!(output_len as usize, input.len(), "level {level}");
                assert_eq!(output, input, "level {level}");
            }
        }
    }

    #[test]
    fn hc_matches_upstream_bytes_for_representative_levels() {
        let cases = [
            (
                12,
                b"the quick brown fox jumps over the lazy dog. "
                    .iter()
                    .copied()
                    .cycle()
                    .take(4096)
                    .collect::<Vec<_>>(),
                "f01074686520717569636b2062726f776e20666f78206a756d7073206f766572201f00af6c617a7920646f672e202d00ffffffffffffffffffffffffffffffca506f672e2074",
            ),
            (
                9,
                patterned_hc_input(128),
                "ff144142434445464741426a6b6c6d6e6f70303132333435363738396162636465666768691a002b144762000d1a00503334353637",
            ),
            (
                10,
                patterned_hc_input(1024),
                "ff144142434445464741426a6b6c6d6e6f70303132333435363738396162636465666768691a002b144762000f4e00320f1a0000144662000fb60039081a00144562000f1e0140011a00144462000f1e0140011a00144362000f1e0140011a00144262000f1e0140011a0005a7020fa4024235333435a7020fa40242356d6e6fa7020fa4024235666768a7020f34001550666768696a",
            ),
        ];

        for (level, input, expected_hex) in cases {
            let expected = decode_hex(expected_hex);
            let mut compressed =
                vec![0u8; unsafe { LZ4_compressBound(input.len() as c_int) } as usize];
            let compressed_len = unsafe {
                LZ4_compress_HC(
                    input.as_ptr() as *const c_char,
                    compressed.as_mut_ptr() as *mut c_char,
                    input.len() as c_int,
                    compressed.len() as c_int,
                    level,
                )
            };
            assert_eq!(compressed_len as usize, expected.len(), "level {level}");
            assert_eq!(
                &compressed[..compressed_len as usize],
                &expected,
                "level {level}"
            );
        }
    }

    #[test]
    fn fast_matches_upstream_bytes_for_representative_blocks() {
        let cases = [
            (
                b"the quick brown fox jumps over the lazy dog. "
                    .iter()
                    .copied()
                    .cycle()
                    .take(4096)
                    .collect::<Vec<_>>(),
                "f01074686520717569636b2062726f776e20666f78206a756d7073206f766572201f00916c617a7920646f672e0e000f2d00ffffffffffffffffffffffffffffffc6506f672e2074",
            ),
            (
                patterned_hc_input(512),
                "ff144142434445464741426a6b6c6d6e6f70303132333435363738396162636465666768691a002b144762000f4e00320f9c000000bd0001c4000fb60039086800011f010062000f1e01400168000281013f4344456c012d014e000fba010000bd0001880109d401506e6f703031",
            ),
        ];

        for (input, expected_hex) in cases {
            let expected = decode_hex(expected_hex);
            let mut compressed =
                vec![0u8; unsafe { LZ4_compressBound(input.len() as c_int) } as usize];
            let compressed_len = unsafe {
                LZ4_compress_default(
                    input.as_ptr() as *const c_char,
                    compressed.as_mut_ptr() as *mut c_char,
                    input.len() as c_int,
                    compressed.len() as c_int,
                )
            };
            assert_eq!(compressed_len as usize, expected.len());
            assert_eq!(&compressed[..compressed_len as usize], &expected);
        }
    }

    #[test]
    fn fast_continue_matches_upstream_bytes_with_dictionary() {
        unsafe {
            let dict = b"abcdefghijklmnop0123456789abcdefghijklmnop0123456789";
            let input = b"abcdefghijklmnop0123456789ZZabcdefghijklmnop0123456789";
            let expected = decode_hex("0f1a00072f5a5a1c0002503536373839");

            let stream = LZ4_createStream();
            assert!(!stream.is_null());
            assert_eq!(
                LZ4_loadDict(stream, dict.as_ptr() as *const c_char, dict.len() as c_int),
                dict.len() as c_int
            );

            let mut compressed = vec![0u8; LZ4_compressBound(input.len() as c_int) as usize];
            let compressed_len = LZ4_compress_fast_continue(
                stream,
                input.as_ptr() as *const c_char,
                compressed.as_mut_ptr() as *mut c_char,
                input.len() as c_int,
                compressed.len() as c_int,
                1,
            );
            LZ4_freeStream(stream);

            assert_eq!(compressed_len as usize, expected.len());
            assert_eq!(&compressed[..compressed_len as usize], &expected);
        }
    }

    #[test]
    fn hc_frame_matches_upstream_bytes_for_representative_input() {
        unsafe {
            let input = patterned_hc_input(1024);
            let cases = [
                (
                    9,
                    "04224d186440a78f000000ff144142434445464741426a6b6c6d6e6f70303132333435363738396162636465666768691a002b144762000e4e000f680033144662000eb6000f680033144562000f1e0140011a00144462002f6869860143144362002f61628601430445023f43333486014305a7022f6d6e86014305a7022f666786014305a7022f383986014305a7020f34001550666768696a00000000e112173a",
                ),
                (
                    10,
                    "04224d186440a796000000ff144142434445464741426a6b6c6d6e6f70303132333435363738396162636465666768691a002b144762000f4e00320f1a0000144662000fb60039081a00144562000f1e0140011a00144462000f1e0140011a00144362000f1e0140011a00144262000f1e0140011a0005a7020fa4024235333435a7020fa40242356d6e6fa7020fa4024235666768a7020f34001550666768696a00000000e112173a",
                ),
                (
                    12,
                    "04224d186440a78f000000ff144142434445464741426a6b6c6d6e6f70303132333435363738396162636465666768691a002b144762000e4e000f680033144662000fb60039081a00144562000f1e0140011a00144462002f6869860143144362002f6162860143144262002f333486014305a7022f6d6e86014305a7022f666786014305a7022f383986014305a7020f34001550666768696a00000000e112173a",
                ),
            ];

            for (level, expected_hex) in cases {
                let expected = decode_hex(expected_hex);
                let prefs = hc_frame_fixture_prefs(level);
                let bound = LZ4F_compressFrameBound(input.len(), &prefs);
                let mut encoded = vec![0u8; bound];
                let encoded_len = LZ4F_compressFrame(
                    encoded.as_mut_ptr() as *mut c_void,
                    encoded.len(),
                    input.as_ptr() as *const c_void,
                    input.len(),
                    &prefs,
                );

                assert_eq!(LZ4F_isError(encoded_len), 0, "level {level}");
                assert_eq!(encoded_len, expected.len(), "level {level}");
                assert_eq!(&encoded[..encoded_len], &expected, "level {level}");
            }
        }
    }

    #[test]
    fn hc_frame_cdict_matches_upstream_bytes() {
        unsafe {
            let dict = b"abcdefghijklmnop0123456789abcdefghijklmnop0123456789";
            let input = b"abcdefghijklmnop0123456789ZZabcdefghijklmnop0123456789";
            let expected = decode_hex(
                "04224d186440a7100000000f1a00072f5a5a1c000250353637383900000000af554433",
            );
            let prefs = hc_frame_fixture_prefs(9);
            let cdict = LZ4F_createCDict(dict.as_ptr() as *const c_void, dict.len());
            assert!(!cdict.is_null());
            let mut cctx = LZ4FCompressionContext(ptr::null_mut());
            assert_eq!(LZ4F_createCompressionContext(&mut cctx, LZ4F_VERSION), 0);
            let mut encoded = vec![0u8; LZ4F_compressFrameBound(input.len(), &prefs)];
            let encoded_len = LZ4F_compressFrame_usingCDict(
                cctx,
                encoded.as_mut_ptr() as *mut c_void,
                encoded.len(),
                input.as_ptr() as *const c_void,
                input.len(),
                cdict,
                &prefs,
            );
            LZ4F_freeCompressionContext(cctx);
            LZ4F_freeCDict(cdict);

            assert_eq!(LZ4F_isError(encoded_len), 0);
            assert_eq!(encoded_len, expected.len());
            assert_eq!(&encoded[..encoded_len], &expected);
        }
    }

    #[test]
    fn hc_multiblock_frame_matches_upstream_hashes() {
        unsafe {
            let input = patterned_hc_input(150_000);
            let cases = [(9, 4504usize, 0x859b_76b8u32), (12, 4504usize, 0x8eb7_3b33u32)];

            for (level, expected_len, expected_hash) in cases {
                let prefs = hc_frame_fixture_prefs(level);
                let mut encoded = vec![0u8; LZ4F_compressFrameBound(input.len(), &prefs)];
                let encoded_len = LZ4F_compressFrame(
                    encoded.as_mut_ptr() as *mut c_void,
                    encoded.len(),
                    input.as_ptr() as *const c_void,
                    input.len(),
                    &prefs,
                );

                assert_eq!(LZ4F_isError(encoded_len), 0, "level {level}");
                assert_eq!(encoded_len, expected_len, "level {level}");
                assert_eq!(
                    xxhash32(&encoded[..encoded_len], 0),
                    expected_hash,
                    "level {level}"
                );
            }
        }
    }

    fn hc_frame_fixture_prefs(level: u32) -> LZ4FPreferences {
        LZ4FPreferences {
            frame_info: LZ4FFrameInfo {
                block_size_id: BlockSize::Max64KB,
                block_mode: BlockMode::Independent,
                content_checksum_flag: ContentChecksum::ChecksumEnabled,
                frame_type: FrameType::Frame,
                content_size: 0,
                dict_id: 0,
                block_checksum_flag: BlockChecksum::NoBlockChecksum,
            },
            compression_level: level,
            auto_flush: 0,
            favor_dec_speed: 0,
            reserved: [0; 3],
        }
    }

    fn patterned_hc_input(len: usize) -> Vec<u8> {
        let pattern = b"abcdefghijklmnop0123456789";
        let mut input = vec![0u8; len];
        for i in 0..len {
            input[i] = pattern[i % pattern.len()];
            if (i % 97) < 9 {
                input[i] = b'A' + (i % 7) as u8;
            }
        }
        input
    }

    fn decode_hex(hex: &str) -> Vec<u8> {
        assert_eq!(hex.len() % 2, 0);
        (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect()
    }

    #[test]
    fn hc_ext_state_dest_size_and_stream_wrappers_round_trip() {
        let input = b"abcdefabcdefabcdefabcdef-".repeat(4096);
        let bound = unsafe { LZ4_compressBound(input.len() as c_int) } as usize;

        let state_size = LZ4_sizeofStateHC();
        assert!(state_size > 0);
        let mut state = vec![0u8; state_size as usize];
        let mut compressed = vec![0u8; bound];
        let compressed_len = unsafe {
            LZ4_compress_HC_extStateHC(
                state.as_mut_ptr() as *mut c_void,
                input.as_ptr() as *const c_char,
                compressed.as_mut_ptr() as *mut c_char,
                input.len() as c_int,
                compressed.len() as c_int,
                9,
            )
        };
        assert!(compressed_len > 0);

        let mut output = vec![0u8; input.len()];
        let output_len = unsafe {
            LZ4_decompress_safe(
                compressed.as_ptr() as *const c_char,
                output.as_mut_ptr() as *mut c_char,
                compressed_len,
                output.len() as c_int,
            )
        };
        assert_eq!(output_len as usize, input.len());
        assert_eq!(output, input);

        let mut source_size = input.len() as c_int;
        let mut tiny = vec![0u8; compressed_len as usize / 2];
        let partial_len = unsafe {
            LZ4_compress_HC_destSize(
                state.as_mut_ptr() as *mut c_void,
                input.as_ptr() as *const c_char,
                tiny.as_mut_ptr() as *mut c_char,
                &mut source_size,
                tiny.len() as c_int,
                9,
            )
        };
        assert!(partial_len > 0);
        assert!(source_size > 0);
        assert!(source_size < input.len() as c_int);

        let stream = unsafe { LZ4_createStreamHC() };
        assert!(!stream.is_null());
        unsafe { LZ4_resetStreamHC_fast(stream, 9) };
        let mut streamed = vec![0u8; bound];
        let streamed_len = unsafe {
            LZ4_compress_HC_continue(
                stream,
                input.as_ptr() as *const c_char,
                streamed.as_mut_ptr() as *mut c_char,
                input.len() as c_int,
                streamed.len() as c_int,
            )
        };
        assert!(streamed_len > 0);
        unsafe { LZ4_freeStreamHC(stream) };

        output.fill(0);
        let output_len = unsafe {
            LZ4_decompress_safe(
                streamed.as_ptr() as *const c_char,
                output.as_mut_ptr() as *mut c_char,
                streamed_len,
                output.len() as c_int,
            )
        };
        assert_eq!(output_len as usize, input.len());
        assert_eq!(output, input);
    }

    #[test]
    fn hc_stream_compression_references_loaded_dictionary() {
        unsafe {
            let dict = b"abcdefghijklmnop";
            let input = b"abcdefghijklmnop";
            let stream = LZ4_createStreamHC();
            assert!(!stream.is_null());
            assert_eq!(
                LZ4_loadDictHC(stream, dict.as_ptr() as *const c_char, dict.len() as c_int),
                dict.len() as c_int
            );

            let mut compressed = vec![0u8; LZ4_compressBound(input.len() as c_int) as usize];
            let compressed_len = LZ4_compress_HC_continue(
                stream,
                input.as_ptr() as *const c_char,
                compressed.as_mut_ptr() as *mut c_char,
                input.len() as c_int,
                compressed.len() as c_int,
            );
            assert!(compressed_len > 0);
            assert!((compressed_len as usize) < input.len() + 1);
            LZ4_freeStreamHC(stream);

            let mut output = vec![0u8; input.len()];
            let output_len = LZ4_decompress_safe_usingDict(
                compressed.as_ptr() as *const c_char,
                output.as_mut_ptr() as *mut c_char,
                compressed_len,
                output.len() as c_int,
                dict.as_ptr() as *const c_char,
                dict.len() as c_int,
            );
            assert_eq!(output_len as usize, input.len());
            assert_eq!(output, input);
        }
    }
}
