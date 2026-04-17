#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::needless_range_loop)]

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
    /// Matches the lz4-rs binding shape. Upstream uses this as a performance
    /// pledge for linked-block history; this pure implementation keeps history
    /// in owned context storage, so the flag is accepted but has no behavioral
    /// effect.
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
const LZ4_HASH_BITS: usize = 12;
const LZ4_HASH_BITS_U16: usize = LZ4_HASH_BITS + 1;
const LZ4_64K_LIMIT: usize = 64 * 1024 + MFLIMIT - 1;
const HASH_UNIT: usize = std::mem::size_of::<usize>();
const LZ4HC_HASH_BITS: usize = 15;
const LZ4HC_HASH_SIZE: usize = 1 << LZ4HC_HASH_BITS;
const LZ4MID_HASH_BITS: usize = LZ4HC_HASH_BITS - 1;
const LZ4MID_HASH_SIZE: usize = 1 << LZ4MID_HASH_BITS;
const LZ4MID_HASHSIZE: usize = 8;
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
const LZ4F_ERROR_MAX_CODE: usize = 24;
const LZ4F_ERROR_GENERIC_CODE: usize = 1;
const LZ4F_ERROR_MAX_BLOCK_SIZE_INVALID_CODE: usize = 2;
const LZ4F_ERROR_PARAMETER_INVALID_CODE: usize = 4;
const LZ4F_ERROR_HEADER_VERSION_WRONG_CODE: usize = 6;
const LZ4F_ERROR_BLOCK_CHECKSUM_INVALID_CODE: usize = 7;
const LZ4F_ERROR_RESERVED_FLAG_SET_CODE: usize = 8;
const LZ4F_ERROR_FRAME_SIZE_WRONG_CODE: usize = 14;
const LZ4F_ERROR_FRAME_TYPE_UNKNOWN_CODE: usize = 13;
const LZ4F_ERROR_SRC_PTR_WRONG_CODE: usize = 15;
const LZ4F_ERROR_DECOMPRESSION_FAILED_CODE: usize = 16;
const LZ4F_ERROR_HEADER_CHECKSUM_INVALID_CODE: usize = 17;
const LZ4F_ERROR_DST_MAX_SIZE_TOO_SMALL_CODE: usize = 11;
const LZ4F_ERROR_FRAME_HEADER_INCOMPLETE_CODE: usize = 12;
const LZ4F_ERROR_CONTENT_CHECKSUM_INVALID_CODE: usize = 18;
const LZ4F_ERROR_FRAME_DECODING_ALREADY_STARTED_CODE: usize = 19;
const LZ4F_ERROR_COMPRESSION_STATE_UNINITIALIZED_CODE: usize = 20;
const LZ4F_ERROR_PARAMETER_NULL_CODE: usize = 21;
const ERROR_GENERIC: usize = lz4f_error(LZ4F_ERROR_GENERIC_CODE);
const ERROR_MAX_BLOCK_SIZE_INVALID: usize = lz4f_error(LZ4F_ERROR_MAX_BLOCK_SIZE_INVALID_CODE);
const ERROR_PARAMETER_INVALID: usize = lz4f_error(LZ4F_ERROR_PARAMETER_INVALID_CODE);
const ERROR_HEADER_VERSION_WRONG: usize = lz4f_error(LZ4F_ERROR_HEADER_VERSION_WRONG_CODE);
const ERROR_BLOCK_CHECKSUM_INVALID: usize = lz4f_error(LZ4F_ERROR_BLOCK_CHECKSUM_INVALID_CODE);
const ERROR_RESERVED_FLAG_SET: usize = lz4f_error(LZ4F_ERROR_RESERVED_FLAG_SET_CODE);
const ERROR_FRAME_SIZE_WRONG: usize = lz4f_error(LZ4F_ERROR_FRAME_SIZE_WRONG_CODE);
const ERROR_FRAME_TYPE_UNKNOWN: usize = lz4f_error(LZ4F_ERROR_FRAME_TYPE_UNKNOWN_CODE);
const ERROR_SRC_PTR_WRONG: usize = lz4f_error(LZ4F_ERROR_SRC_PTR_WRONG_CODE);
const ERROR_DECOMPRESSION_FAILED: usize = lz4f_error(LZ4F_ERROR_DECOMPRESSION_FAILED_CODE);
const ERROR_HEADER_CHECKSUM_INVALID: usize = lz4f_error(LZ4F_ERROR_HEADER_CHECKSUM_INVALID_CODE);
const ERROR_DST_TOO_SMALL: usize = lz4f_error(LZ4F_ERROR_DST_MAX_SIZE_TOO_SMALL_CODE);
const ERROR_BAD_HEADER: usize = lz4f_error(LZ4F_ERROR_FRAME_HEADER_INCOMPLETE_CODE);
const ERROR_CHECKSUM_INVALID: usize = lz4f_error(LZ4F_ERROR_CONTENT_CHECKSUM_INVALID_CODE);
const ERROR_FRAME_DECODING_ALREADY_STARTED: usize =
    lz4f_error(LZ4F_ERROR_FRAME_DECODING_ALREADY_STARTED_CODE);
const ERROR_COMPRESSION_STATE_UNINITIALIZED: usize =
    lz4f_error(LZ4F_ERROR_COMPRESSION_STATE_UNINITIALIZED_CODE);
const ERROR_PARAMETER_NULL: usize = lz4f_error(LZ4F_ERROR_PARAMETER_NULL_CODE);

static ERROR_GENERIC_NAME: &[u8] = b"ERROR_GENERIC\0";
static ERROR_MAX_BLOCK_SIZE_NAME: &[u8] = b"ERROR_maxBlockSize_invalid\0";
static ERROR_PARAMETER_INVALID_NAME: &[u8] = b"ERROR_parameter_invalid\0";
static ERROR_HEADER_VERSION_NAME: &[u8] = b"ERROR_headerVersion_wrong\0";
static ERROR_BLOCK_CHECKSUM_NAME: &[u8] = b"ERROR_blockChecksum_invalid\0";
static ERROR_RESERVED_FLAG_NAME: &[u8] = b"ERROR_reservedFlag_set\0";
static ERROR_FRAME_SIZE_NAME: &[u8] = b"ERROR_frameSize_wrong\0";
static ERROR_FRAME_TYPE_NAME: &[u8] = b"ERROR_frameType_unknown\0";
static ERROR_SRC_PTR_NAME: &[u8] = b"ERROR_srcPtr_wrong\0";
static ERROR_DECOMPRESSION_FAILED_NAME: &[u8] = b"ERROR_decompressionFailed\0";
static ERROR_HEADER_CHECKSUM_NAME: &[u8] = b"ERROR_headerChecksum_invalid\0";
static ERROR_DST_NAME: &[u8] = b"ERROR_dstMaxSize_tooSmall\0";
static ERROR_BAD_HEADER_NAME: &[u8] = b"ERROR_frameHeader_incomplete\0";
static ERROR_CHECKSUM_NAME: &[u8] = b"ERROR_contentChecksum_invalid\0";
static ERROR_FRAME_DECODING_ALREADY_STARTED_NAME: &[u8] = b"ERROR_frameDecoding_alreadyStarted\0";
static ERROR_COMPRESSION_STATE_UNINITIALIZED_NAME: &[u8] =
    b"ERROR_compressionState_uninitialized\0";
static ERROR_PARAMETER_NULL_NAME: &[u8] = b"ERROR_parameter_null\0";
static ERROR_UNSPECIFIED_NAME: &[u8] = b"Unspecified error code\0";
static LZ4_VERSION_STRING_BYTES: &[u8] = b"1.10.0\0";

const fn lz4f_error(code: usize) -> usize {
    usize::MAX - (code - 1)
}

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
    dict_id: c_uint,
    compression_level: c_int,
    favor_dec_speed: bool,
}

impl Default for FramePrefs {
    fn default() -> Self {
        Self {
            block_size_id: 4,
            block_independent: true,
            block_checksum: false,
            content_checksum: false,
            content_size: 0,
            dict_id: 0,
            compression_level: 0,
            favor_dec_speed: false,
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
    dict_id: c_uint,
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
    attached_dictionary: bool,
    favor_dec_speed: bool,
}

#[derive(Debug, Default)]
struct EncodeStreamCtx {
    dictionary: Vec<u8>,
    attached_dictionary: bool,
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
            attached_dictionary: false,
            favor_dec_speed: false,
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
            dict_id: 0,
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
    if !(0..=LZ4_MAX_INPUT_SIZE).contains(&size) {
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
    acceleration: c_int,
) -> c_int {
    if sourceSize < 0 || maxDestSize <= 0 || source.is_null() || dest.is_null() {
        return 0;
    }
    let src = slice::from_raw_parts(source as *const u8, sourceSize as usize);
    let dst = slice::from_raw_parts_mut(dest as *mut u8, maxDestSize as usize);
    compress_block(src, dst, normalize_acceleration(acceleration)).map_or(0, |n| n as c_int)
}

#[no_mangle]
pub extern "C" fn LZ4_sizeofState() -> c_int {
    ((1usize << (LZ4_HASH_BITS + 2)) + 32) as c_int
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
    compress_block_hc(src, dst, compressionLevel, false).map_or(0, |n| n as c_int)
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
    let Some((consumed, written)) =
        compress_hc_dest_size(src_slice, dst_slice, compressionLevel, false)
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
    if !(0..=LZ4_MAX_INPUT_SIZE).contains(&maxBlockSize) {
        return 0;
    }
    let max_block_size = cmp::max(maxBlockSize, 16);
    max_block_size.saturating_add((LZ4_DISTANCE_MAX + 1 + 14) as c_int)
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
    (code > lz4f_error(LZ4F_ERROR_MAX_CODE)) as c_uint
}

#[no_mangle]
pub extern "C" fn LZ4F_getErrorName(code: size_t) -> *const c_char {
    match code {
        ERROR_MAX_BLOCK_SIZE_INVALID => ERROR_MAX_BLOCK_SIZE_NAME.as_ptr() as *const c_char,
        ERROR_PARAMETER_INVALID => ERROR_PARAMETER_INVALID_NAME.as_ptr() as *const c_char,
        ERROR_HEADER_VERSION_WRONG => ERROR_HEADER_VERSION_NAME.as_ptr() as *const c_char,
        ERROR_BLOCK_CHECKSUM_INVALID => ERROR_BLOCK_CHECKSUM_NAME.as_ptr() as *const c_char,
        ERROR_RESERVED_FLAG_SET => ERROR_RESERVED_FLAG_NAME.as_ptr() as *const c_char,
        ERROR_FRAME_SIZE_WRONG => ERROR_FRAME_SIZE_NAME.as_ptr() as *const c_char,
        ERROR_FRAME_TYPE_UNKNOWN => ERROR_FRAME_TYPE_NAME.as_ptr() as *const c_char,
        ERROR_SRC_PTR_WRONG => ERROR_SRC_PTR_NAME.as_ptr() as *const c_char,
        ERROR_DECOMPRESSION_FAILED => ERROR_DECOMPRESSION_FAILED_NAME.as_ptr() as *const c_char,
        ERROR_HEADER_CHECKSUM_INVALID => ERROR_HEADER_CHECKSUM_NAME.as_ptr() as *const c_char,
        ERROR_DST_TOO_SMALL => ERROR_DST_NAME.as_ptr() as *const c_char,
        ERROR_BAD_HEADER => ERROR_BAD_HEADER_NAME.as_ptr() as *const c_char,
        ERROR_CHECKSUM_INVALID => ERROR_CHECKSUM_NAME.as_ptr() as *const c_char,
        ERROR_FRAME_DECODING_ALREADY_STARTED => {
            ERROR_FRAME_DECODING_ALREADY_STARTED_NAME.as_ptr() as *const c_char
        }
        ERROR_COMPRESSION_STATE_UNINITIALIZED => {
            ERROR_COMPRESSION_STATE_UNINITIALIZED_NAME.as_ptr() as *const c_char
        }
        ERROR_PARAMETER_NULL => ERROR_PARAMETER_NULL_NAME.as_ptr() as *const c_char,
        ERROR_GENERIC => ERROR_GENERIC_NAME.as_ptr() as *const c_char,
        _ => ERROR_UNSPECIFIED_NAME.as_ptr() as *const c_char,
    }
}

#[no_mangle]
pub extern "C" fn LZ4F_getErrorCode(code: size_t) -> c_uint {
    if LZ4F_isError(code) == 0 {
        0
    } else {
        (usize::MAX - code + 1) as c_uint
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
        return ERROR_MAX_BLOCK_SIZE_INVALID;
    }
    block_max_size(blockSizeID as u8)
}

#[no_mangle]
pub unsafe extern "C" fn LZ4F_createCompressionContext(
    ctx: &mut LZ4FCompressionContext,
    version: c_uint,
) -> LZ4FErrorCode {
    if version != LZ4F_VERSION {
        return ERROR_PARAMETER_INVALID;
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
        return ERROR_PARAMETER_NULL;
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
        return ERROR_SRC_PTR_WRONG;
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
        return ERROR_PARAMETER_NULL;
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
        return ERROR_PARAMETER_NULL;
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
        return ERROR_PARAMETER_NULL;
    }
    let inner = &mut *(ctx.0 as *mut CompressionCtx);
    if !inner.started {
        return ERROR_COMPRESSION_STATE_UNINITIALIZED;
    }
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
            let Some(remaining_dst) = dst.get_mut(written..) else {
                return ERROR_DST_TOO_SMALL;
            };
            let n = compress_frame_update_block(inner, chunk, remaining_dst);
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

fn compress_frame_update_block(inner: &mut CompressionCtx, src: &[u8], dst: &mut [u8]) -> size_t {
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
    if inner.prefs.content_checksum {
        inner.content_hasher.update(src);
    }
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
        return ERROR_PARAMETER_NULL;
    }
    let inner = &mut *(ctx.0 as *mut CompressionCtx);
    if !inner.started {
        return ERROR_COMPRESSION_STATE_UNINITIALIZED;
    }
    let src = if srcSize > 0 {
        slice::from_raw_parts(srcBuffer as *const u8, srcSize)
    } else {
        &[]
    };
    let dst = slice::from_raw_parts_mut(dstBuffer as *mut u8, dstCapacity);
    let block_max = block_max_size(inner.prefs.block_size_id);
    let mut written = 0usize;
    for chunk in src.chunks(block_max) {
        let Some(remaining_dst) = dst.get_mut(written..) else {
            return ERROR_DST_TOO_SMALL;
        };
        let n = compress_frame_raw_block(inner, chunk, remaining_dst);
        if LZ4F_isError(n) != 0 {
            return n;
        }
        written += n;
    }
    written
}

fn compress_frame_raw_block(inner: &mut CompressionCtx, src: &[u8], dst: &mut [u8]) -> size_t {
    let checksum_len = if inner.prefs.block_checksum { 4 } else { 0 };
    let needed = 4 + src.len() + checksum_len;
    if dst.len() < needed {
        return ERROR_DST_TOO_SMALL;
    }
    let block_size = (src.len() as u32) | 0x8000_0000;
    dst[..4].copy_from_slice(&block_size.to_le_bytes());
    dst[4..4 + src.len()].copy_from_slice(src);
    if inner.prefs.block_checksum {
        let checksum = xxhash32(&dst[4..4 + src.len()], 0);
        dst[4 + src.len()..needed].copy_from_slice(&checksum.to_le_bytes());
    }
    if inner.prefs.content_checksum {
        inner.content_hasher.update(src);
    }
    if !inner.prefs.block_independent {
        append_hc_dictionary(&mut inner.dictionary, src);
    } else if inner.external_dictionary {
        inner.dictionary.clear();
        inner.external_dictionary = false;
    }
    needed
}

#[no_mangle]
pub unsafe extern "C" fn LZ4F_flush(
    ctx: LZ4FCompressionContext,
    dstBuffer: *mut u8,
    _dstCapacity: size_t,
    _cOptPtr: *const LZ4FCompressOptions,
) -> size_t {
    if ctx.0.is_null() || dstBuffer.is_null() {
        return ERROR_PARAMETER_NULL;
    }
    let inner = &mut *(ctx.0 as *mut CompressionCtx);
    if !inner.started {
        return ERROR_COMPRESSION_STATE_UNINITIALIZED;
    }
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
        return ERROR_PARAMETER_NULL;
    }
    let inner = &mut *(ctx.0 as *mut CompressionCtx);
    if !inner.started {
        return ERROR_COMPRESSION_STATE_UNINITIALIZED;
    }
    let needed = 4 + if inner.prefs.content_checksum { 4 } else { 0 };
    if dstCapacity < needed {
        return ERROR_DST_TOO_SMALL;
    }
    let dst = slice::from_raw_parts_mut(dstBuffer, dstCapacity);
    dst[..4].copy_from_slice(&0u32.to_le_bytes());
    if inner.prefs.content_checksum {
        dst[4..8].copy_from_slice(&inner.content_hasher.digest().to_le_bytes());
    }
    inner.started = false;
    needed
}

#[no_mangle]
pub unsafe extern "C" fn LZ4F_createDecompressionContext(
    ctx: &mut LZ4FDecompressionContext,
    version: c_uint,
) -> LZ4FErrorCode {
    if version != LZ4F_VERSION {
        return ERROR_PARAMETER_INVALID;
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
        return ERROR_PARAMETER_NULL;
    }
    let inner = &mut *(ctx.0 as *mut DecompressionCtx);
    if inner.parsed_header {
        *srcSizePtr = 0;
        *frameInfoPtr = frame_info_from_decompression_ctx(inner);
        return frame_hint(inner);
    }
    if !inner.input.is_empty() {
        *srcSizePtr = 0;
        return ERROR_FRAME_DECODING_ALREADY_STARTED;
    }
    let src_size = *srcSizePtr;
    if srcBuffer.is_null() {
        *srcSizePtr = 0;
        return ERROR_SRC_PTR_WRONG;
    }
    if src_size < 7 {
        *srcSizePtr = 0;
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
    let (prefs, header_len) = match parse_frame_header(src) {
        Ok(parsed) => parsed,
        Err(code) => {
            *srcSizePtr = 0;
            return code;
        }
    };
    if !inner.parsed_header && !inner.done {
        apply_frame_prefs(inner, prefs);
    }
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
        dict_id: prefs.dict_id,
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
    if src.is_null() {
        return ERROR_SRC_PTR_WRONG;
    }
    if srcSize < 5 {
        return ERROR_BAD_HEADER;
    }
    let src = slice::from_raw_parts(src as *const u8, srcSize);
    if is_skippable_magic_prefix(src) {
        return 8;
    }
    if src[..4] != LZ4F_MAGIC {
        return ERROR_FRAME_TYPE_UNKNOWN;
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
    dOptPtr: *const LZ4FDecompressOptions,
) -> size_t {
    if ctx.0.is_null() || dstSizePtr.is_null() || srcSizePtr.is_null() {
        return ERROR_PARAMETER_NULL;
    }
    // `stable_dst` is an upstream storage optimization. This implementation
    // keeps linked-block history in `DecompressionCtx::dictionary`, so both
    // stable and unstable destination modes share the same behavior.
    let _stable_dst = !dOptPtr.is_null() && (*dOptPtr).stable_dst != 0;
    let inner = &mut *(ctx.0 as *mut DecompressionCtx);
    let src_size = *srcSizePtr;
    let dst_capacity = *dstSizePtr;
    if dst_capacity > 0 && dstBuffer.is_null() {
        return ERROR_PARAMETER_NULL;
    }

    if src_size > 0
        && dst_capacity > 0
        && !srcBuffer.is_null()
        && inner.input.is_empty()
        && pending_is_empty(inner)
        && inner.parsed_header
        && !inner.done
    {
        let src = slice::from_raw_parts(srcBuffer, src_size);
        if let Some(result) = try_decompress_frame_block_slice_to_dst(
            inner,
            src,
            slice::from_raw_parts_mut(dstBuffer, dst_capacity),
        ) {
            match result {
                Ok((consumed, written)) => {
                    *srcSizePtr = consumed;
                    *dstSizePtr = written;
                    if inner.done && pending_is_empty(inner) {
                        *inner = DecompressionCtx::default();
                        return 0;
                    }
                    return frame_hint(inner);
                }
                Err(code) => return code,
            }
        }
    }

    if src_size > 0 {
        if srcBuffer.is_null() {
            return ERROR_SRC_PTR_WRONG;
        }
        inner
            .input
            .extend_from_slice(slice::from_raw_parts(srcBuffer, src_size));
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
                    let consumed = consumed_from_call(inner.done, src_size, inner.input.len());
                    *srcSizePtr = consumed;
                    *dstSizePtr = written;
                    if inner.done && pending_is_empty(inner) {
                        *inner = DecompressionCtx::default();
                        return 0;
                    }
                    return frame_hint(inner);
                }
                Err(code) => return code,
            }
        }
    }

    if let Err(code) = parse_available_frame(inner) {
        return code;
    }
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
    *srcSizePtr = consumed_from_call(inner.done, src_size, inner.input.len());
    if inner.done && pending_is_empty(inner) {
        *inner = DecompressionCtx::default();
        return 0;
    }
    frame_hint(inner)
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
        return ERROR_PARAMETER_NULL;
    }
    if dictSize > 0 && dict.is_null() {
        return ERROR_SRC_PTR_WRONG;
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
        compress_block(src_slice, dst_slice, 1)
    } else {
        compress_block_with_dict(src_slice, dst_slice, &ctx.dictionary, 1)
    }
    .map_or(0, |n| n as c_int);
    if written > 0 {
        if ctx.attached_dictionary {
            ctx.dictionary.clear();
            append_hc_dictionary(&mut ctx.dictionary, src_slice);
            ctx.attached_dictionary = false;
        } else {
            append_hc_dictionary(&mut ctx.dictionary, src_slice);
        }
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
    acceleration: c_int,
) -> c_int {
    if stream.is_null() || srcSize < 0 || dstCapacity <= 0 || src.is_null() || dst.is_null() {
        return 0;
    }
    let ctx = &mut *(stream as *mut EncodeStreamCtx);
    let src_slice = slice::from_raw_parts(src as *const u8, srcSize as usize);
    let dst_slice = slice::from_raw_parts_mut(dst as *mut u8, dstCapacity as usize);
    let acceleration = normalize_acceleration(acceleration);
    let written = if ctx.dictionary.is_empty() {
        compress_block(src_slice, dst_slice, acceleration)
    } else {
        compress_block_with_dict(src_slice, dst_slice, &ctx.dictionary, acceleration)
    }
    .map_or(0, |n| n as c_int);
    if written > 0 {
        if ctx.attached_dictionary {
            ctx.dictionary.clear();
            append_hc_dictionary(&mut ctx.dictionary, src_slice);
            ctx.attached_dictionary = false;
        } else {
            append_hc_dictionary(&mut ctx.dictionary, src_slice);
        }
    }
    written
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
        let ctx = &mut *(stream as *mut EncodeStreamCtx);
        ctx.dictionary.clear();
        ctx.attached_dictionary = false;
    }
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_resetStream(stream: *mut LZ4StreamEncode) {
    LZ4_resetStream_fast(stream)
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_initStream(
    stateBuffer: *mut c_void,
    size: size_t,
) -> *mut LZ4StreamEncode {
    if stateBuffer.is_null() {
        return ptr::null_mut();
    }
    if size < LZ4_sizeofStreamState() as usize {
        return ptr::null_mut();
    }
    if !(stateBuffer as usize).is_multiple_of(std::mem::align_of::<EncodeStreamCtx>()) {
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
pub unsafe extern "C" fn LZ4_slideInputBuffer(state: *mut c_void) -> *mut c_char {
    if state.is_null() {
        return ptr::null_mut();
    }
    let ctx = &mut *(state as *mut EncodeStreamCtx);
    if ctx.dictionary.is_empty() {
        ptr::null_mut()
    } else {
        ctx.dictionary.as_mut_ptr() as *mut c_char
    }
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
    working.attached_dictionary = false;
    if !dictionaryStream.is_null() {
        let dictionary = &*(dictionaryStream as *const EncodeStreamCtx);
        working.dictionary.extend_from_slice(&dictionary.dictionary);
        working.attached_dictionary = !working.dictionary.is_empty();
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
    ctx.attached_dictionary = false;
    if dictSize as usize >= HASH_UNIT {
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
        ctx.attached_dictionary = false;
    } else {
        ctx.dictionary.clear();
        ctx.attached_dictionary = false;
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
        ctx.attached_dictionary = false;
    }
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_resetStreamHC(stream: *mut LZ4StreamHC, compressionLevel: c_int) {
    LZ4_resetStreamHC_fast(stream, compressionLevel)
}

#[no_mangle]
pub unsafe extern "C" fn LZ4_initStreamHC(buffer: *mut c_void, size: size_t) -> *mut LZ4StreamHC {
    if buffer.is_null() {
        return ptr::null_mut();
    }
    if size < LZ4_sizeofStreamStateHC() as usize {
        return ptr::null_mut();
    }
    if !(buffer as usize).is_multiple_of(std::mem::align_of::<HcStreamCtx>()) {
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
pub unsafe extern "C" fn LZ4_favorDecompressionSpeed(stream: *mut LZ4StreamHC, favor: c_int) {
    if !stream.is_null() {
        (*(stream as *mut HcStreamCtx)).favor_dec_speed = favor != 0;
    }
}

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
    working.attached_dictionary = false;
    if !dictionary_stream.is_null() {
        let dictionary = &*(dictionary_stream as *const HcStreamCtx);
        working.dictionary.extend_from_slice(&dictionary.dictionary);
        working.attached_dictionary = !working.dictionary.is_empty();
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
    ctx.attached_dictionary = false;
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
    if maxDictSize < 4 {
        ctx.dictionary.clear();
        ctx.attached_dictionary = false;
        return 0;
    }
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
        ctx.attached_dictionary = false;
    } else {
        ctx.dictionary.clear();
        ctx.attached_dictionary = false;
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
    compress_hc_stream_block(
        ctx,
        src_slice,
        dst_slice,
        ctx.compression_level,
        ctx.favor_dec_speed,
    )
}

fn compress_hc_stream_block(
    ctx: &mut HcStreamCtx,
    src: &[u8],
    dst: &mut [u8],
    compression_level: c_int,
    favor_dec_speed: bool,
) -> c_int {
    let written = if ctx.dictionary.is_empty() {
        compress_block_hc(src, dst, compression_level, favor_dec_speed)
    } else {
        compress_block_hc_with_dict(
            src,
            dst,
            &ctx.dictionary,
            compression_level,
            favor_dec_speed,
        )
    }
    .map_or(0, |n| n as c_int);
    if written > 0 && !src.is_empty() {
        if ctx.attached_dictionary {
            ctx.dictionary.clear();
            append_hc_dictionary(&mut ctx.dictionary, src);
            ctx.attached_dictionary = false;
        } else {
            append_hc_dictionary(&mut ctx.dictionary, src);
        }
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
        compress_hc_dest_size(
            src_slice,
            dst_slice,
            ctx.compression_level,
            ctx.favor_dec_speed,
        )
    } else {
        compress_hc_dest_size_with_dict(
            src_slice,
            dst_slice,
            &ctx.dictionary,
            ctx.compression_level,
            ctx.favor_dec_speed,
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
        if ctx.attached_dictionary {
            ctx.dictionary.clear();
            append_hc_dictionary(&mut ctx.dictionary, src_slice);
            ctx.attached_dictionary = false;
        } else {
            append_hc_dictionary(&mut ctx.dictionary, src_slice);
        }
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
    if srcSize < 0 || src.is_null() || dst.is_null() {
        return 0;
    }
    let bound = LZ4_compressBound(srcSize);
    if bound <= 0 {
        return 0;
    }
    let ctx = &mut *(stream as *mut HcStreamCtx);
    let src_slice = slice::from_raw_parts(src as *const u8, srcSize as usize);
    let dst_slice = slice::from_raw_parts_mut(dst as *mut u8, bound as usize);
    compress_hc_stream_block(
        ctx,
        src_slice,
        dst_slice,
        normalize_hc_level(cLevel),
        ctx.favor_dec_speed,
    )
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
    if srcSize < 0 || maxDstSize <= 0 || src.is_null() || dst.is_null() {
        return 0;
    }
    let ctx = &mut *(stream as *mut HcStreamCtx);
    let src_slice = slice::from_raw_parts(src as *const u8, srcSize as usize);
    let dst_slice = slice::from_raw_parts_mut(dst as *mut u8, maxDstSize as usize);
    compress_hc_stream_block(
        ctx,
        src_slice,
        dst_slice,
        normalize_hc_level(cLevel),
        ctx.favor_dec_speed,
    )
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
pub unsafe extern "C" fn LZ4_slideInputBufferHC(stream: *mut c_void) -> *mut c_char {
    if stream.is_null() {
        return ptr::null_mut();
    }
    let ctx = &mut *(stream as *mut HcStreamCtx);
    if ctx.dictionary.is_empty() {
        ptr::null_mut()
    } else {
        let ptr = ctx.dictionary.as_mut_ptr() as *mut c_char;
        ctx.dictionary.clear();
        ctx.attached_dictionary = false;
        ptr
    }
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

fn normalize_acceleration(acceleration: c_int) -> usize {
    cmp::max(acceleration, 1) as usize
}

fn compress_block(src: &[u8], dst: &mut [u8], acceleration: usize) -> Option<usize> {
    if src.is_empty() {
        return emit_last_literals(src, dst, 0, 0);
    }
    if src.len() < MFLIMIT + 1 {
        return emit_last_literals(src, dst, 0, 0);
    }
    let by_u16 = src.len() < LZ4_64K_LIMIT;
    let hash_bits = if by_u16 {
        LZ4_HASH_BITS_U16
    } else {
        LZ4_HASH_BITS
    };
    let mut table = vec![0usize; 1 << hash_bits];
    let mut ip = 0usize;
    let mut anchor = 0usize;
    let mut op = 0usize;
    let mflimit_plus_one = src.len() - MFLIMIT + 1;
    let match_limit = src.len() - LAST_LITERALS;

    table[hash_fast(src, ip, by_u16)] = ip;
    ip += 1;
    let mut forward_h = hash_fast(src, ip, by_u16);

    loop {
        let mut forward_ip = ip;
        let mut step = 1usize;
        let mut search_match_nb = acceleration << 6;
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
            forward_h = hash_fast(src, forward_ip, by_u16);
            table[h] = ip;

            if ip > ref_pos
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

            table[hash_fast(src, ip - 2, by_u16)] = ip - 2;
            let h = hash_fast(src, ip, by_u16);
            ref_pos = table[h];
            table[h] = ip;
            if ip > ref_pos
                && ip - ref_pos <= LZ4_DISTANCE_MAX
                && src[ref_pos..ref_pos + MINMATCH] == src[ip..ip + MINMATCH]
            {
                continue;
            }

            ip += 1;
            forward_h = hash_fast(src, ip, by_u16);
            break;
        }
    }
}

fn compress_dest_size(src: &[u8], dst: &mut [u8]) -> Option<(usize, usize)> {
    if src.is_empty() {
        let written = compress_block(src, dst, 1)?;
        return Some((0, written));
    }

    let mut low = 0usize;
    let mut high = src.len();
    let mut best: Option<(usize, usize, Vec<u8>)> = None;
    while low <= high {
        let mid = low + (high - low) / 2;
        let mut candidate = vec![0u8; dst.len()];
        match compress_block(&src[..mid], &mut candidate, 1) {
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

fn compress_block_with_dict(
    src: &[u8],
    dst: &mut [u8],
    dict: &[u8],
    acceleration: usize,
) -> Option<usize> {
    if src.is_empty() {
        return emit_last_literals(src, dst, 0, 0);
    }
    if src.len() < MFLIMIT + 1 {
        return emit_last_literals(src, dst, 0, 0);
    }
    let dict_keep = cmp::min(dict.len(), LZ4_DISTANCE_MAX);
    let dict = &dict[dict.len() - dict_keep..];
    if dict.is_empty() {
        return compress_block(src, dst, acceleration);
    }

    let mut full = Vec::with_capacity(dict.len() + src.len());
    full.extend_from_slice(dict);
    full.extend_from_slice(src);
    let base = dict.len();
    let mut table = vec![0usize; 1 << LZ4_HASH_BITS];
    let seed_end = full.len().saturating_sub(MINMATCH - 1);
    for pos in 0..cmp::min(base, seed_end) {
        table[hash_fast(&full, pos, false)] = pos;
    }

    let mut ip = base;
    let mut anchor = base;
    let mut op = 0usize;
    let mflimit_plus_one = base + src.len() - MFLIMIT + 1;
    let match_limit = base + src.len() - LAST_LITERALS;

    table[hash_fast(&full, ip, false)] = ip;
    ip += 1;
    let mut forward_h = hash_fast(&full, ip, false);

    loop {
        let mut forward_ip = ip;
        let mut step = 1usize;
        let mut search_match_nb = acceleration << 6;
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
            forward_h = hash_fast(&full, forward_ip, false);
            table[h] = ip;

            if ip > ref_pos
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
            op = encode_sequence(&full, dst, anchor, ip, match_len, ip - ref_pos, op)?;
            ip += match_len;
            anchor = ip;

            if ip >= mflimit_plus_one {
                return emit_last_literals_with_base(&full, dst, base, anchor, op);
            }

            table[hash_fast(&full, ip - 2, false)] = ip - 2;
            let h = hash_fast(&full, ip, false);
            ref_pos = table[h];
            table[h] = ip;
            if ip > ref_pos
                && ip - ref_pos <= LZ4_DISTANCE_MAX
                && full[ref_pos..ref_pos + MINMATCH] == full[ip..ip + MINMATCH]
            {
                continue;
            }

            ip += 1;
            forward_h = hash_fast(&full, ip, false);
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

/// Starting offset added to every physical position before it is stored in
/// `HcTables::hash` or used for chain arithmetic. Mirrors upstream's
/// `LZ4HC_init_internal` which pushes `dictLimit` and `lowLimit` to at least
/// `64 KiB` so the `u16`-delta chain walk can never wrap a valid candidate
/// below zero: any logical position stored in the hash table is `>= 64 KiB`
/// and every chain delta is `<= 65535`, so `candidate_log -= delta` stays
/// positive and the `candidate_log >= lowest_log` exit is exact.
const HC_OFFSET: u32 = (LZ4_DISTANCE_MAX + 1) as u32;

#[derive(Debug)]
struct HcTables {
    /// Head-of-chain per hash: logical positions (`physical + HC_OFFSET`).
    /// Initialized to `0` so an empty bucket yields a `candidate_log < HC_OFFSET`
    /// that fails the `candidate_log >= lowest_log` loop condition without any
    /// explicit sentinel check.
    hash: Vec<u32>,
    /// Delta to the previous chain entry, clamped to `LZ4_DISTANCE_MAX`.
    /// Initial `0xFFFF` entries mirror upstream's `MEM_INIT(chainTable, 0xFF)`
    /// so first-ever walks step off the end of the search window immediately.
    chain: Vec<u16>,
    next_to_update: usize,
    /// Offset inside the search buffer where the current prefix begins.
    /// For no-dict compression this is `0`; when a dictionary is prepended
    /// before the input, it is `dict.len()`. Equivalent to upstream's
    /// `prefixPtr` in that `prefixPtr == &full[base]`. Used by the HC
    /// pattern-analysis branch to replicate upstream's `protectDictEnd`
    /// guard so the repeated-pattern fast path is not entered when a
    /// dict-area candidate sits in the last 3 bytes before the prefix.
    base: usize,
}

impl HcTables {
    fn with_base(_src_len: usize, base: usize) -> Self {
        Self {
            hash: vec![0u32; LZ4HC_HASH_SIZE],
            chain: vec![LZ4_DISTANCE_MAX as u16; LZ4_DISTANCE_MAX + 1],
            next_to_update: 0,
            base,
        }
    }

    /// Convert a physical buffer position to the logical position used for
    /// storage in `hash` and for chain arithmetic.
    #[inline(always)]
    fn to_log(pos: usize) -> u32 {
        (pos as u32).wrapping_add(HC_OFFSET)
    }

    /// Convert a logical position back to the physical buffer position. Only
    /// valid after a `>= lowest_log` check has ensured the logical position
    /// is at least `HC_OFFSET`.
    #[inline(always)]
    fn to_phys(log: u32) -> usize {
        log.wrapping_sub(HC_OFFSET) as usize
    }

    #[inline]
    fn insert_until(&mut self, src: &[u8], target: usize) {
        let end = cmp::min(target, src.len().saturating_sub(MINMATCH - 1));
        while self.next_to_update < end {
            let pos = self.next_to_update;
            let h = hash4_hc(src, pos);
            let log_pos = Self::to_log(pos);
            let prev_log = self.hash[h];
            // delta = log_pos - prev_log, clamped to LZ4_DISTANCE_MAX. When
            // `prev_log` is 0 (empty bucket) the computed delta always exceeds
            // 65535, so it clamps to the sentinel and a later chain walk
            // lands below `lowest_log` on the next step.
            let delta = cmp::min(log_pos - prev_log, HC_OFFSET - 1) as u16;
            self.chain[pos & LZ4_DISTANCE_MAX] = delta;
            self.hash[h] = log_pos;
            self.next_to_update += 1;
        }
    }

    /// Returns the physical position stored in the chain before `pos`, or
    /// `usize::MAX` if the chain step goes below the HC search window (the
    /// logical delta lands at or below `HC_OFFSET`).
    #[inline]
    fn previous(&self, pos: usize) -> usize {
        let log_pos = Self::to_log(pos);
        let delta = self.chain[pos & LZ4_DISTANCE_MAX] as u32;
        let prev_log = log_pos.wrapping_sub(delta);
        if prev_log < HC_OFFSET {
            usize::MAX
        } else {
            Self::to_phys(prev_log)
        }
    }
}

struct MidTables {
    hash4: Vec<usize>,
    hash8: Vec<usize>,
}

impl MidTables {
    fn new() -> Self {
        Self {
            hash4: vec![usize::MAX; LZ4MID_HASH_SIZE],
            hash8: vec![usize::MAX; LZ4MID_HASH_SIZE],
        }
    }

    fn add4(&mut self, src: &[u8], pos: usize) {
        self.add4_index(src, pos, pos);
    }

    fn add4_index(&mut self, src: &[u8], pos: usize, index: usize) {
        if pos + MINMATCH <= src.len() {
            self.hash4[hash4_mid(src, pos)] = index;
        }
    }

    fn add8(&mut self, src: &[u8], pos: usize) {
        self.add8_index(src, pos, pos);
    }

    fn add8_index(&mut self, src: &[u8], pos: usize, index: usize) {
        if pos + LZ4MID_HASHSIZE <= src.len() {
            self.hash8[hash8_mid(src, pos)] = index;
        }
    }
}

fn fill_lz4mid_dict_table(table: &mut MidTables, full: &[u8], base: usize) {
    if base <= LZ4MID_HASHSIZE {
        return;
    }
    let target = base - LZ4MID_HASHSIZE;
    let mut idx = 0usize;
    while idx < target {
        table.add4_index(full, idx, idx);
        table.add8_index(full, idx + 1, idx + 1);
        idx += 3;
    }

    idx = if base > 32 * 1024 + LZ4MID_HASHSIZE {
        target - 32 * 1024
    } else {
        0
    };
    while idx < target {
        table.add8_index(full, idx, idx);
        idx += 1;
    }
}

fn compress_block_lz4mid(src: &[u8], dst: &mut [u8]) -> Option<usize> {
    compress_block_lz4mid_with_base(src, dst, 0)
}

fn compress_block_lz4mid_with_base(full: &[u8], dst: &mut [u8], base: usize) -> Option<usize> {
    let src_len = full.len().checked_sub(base)?;
    if src_len == 0 {
        return emit_last_literals_with_base(full, dst, base, base, 0);
    }
    if src_len < MFLIMIT + 1 {
        return emit_last_literals_with_base(full, dst, base, base, 0);
    }

    let mut table = MidTables::new();
    if base > 0 {
        fill_lz4mid_dict_table(&mut table, full, base);
    }
    let mut anchor = base;
    let mut ip = base;
    let mut op = 0usize;
    let mflimit = full.len() - MFLIMIT;
    let match_limit = full.len() - LAST_LITERALS;

    while ip <= mflimit {
        let search_ip = ip;
        let mut match_len = 0usize;
        let mut match_distance = 0usize;

        let h8 = hash8_mid(full, ip);
        let pos8 = table.hash8[h8];
        table.hash8[h8] = ip;
        if pos8 != usize::MAX
            && ip - pos8 <= LZ4_DISTANCE_MAX
            && full[pos8..pos8 + MINMATCH] == full[ip..ip + MINMATCH]
        {
            match_len = count_match(full, ip, pos8, match_limit);
            if match_len >= MINMATCH {
                match_distance = ip - pos8;
            }
        }

        if match_len < MINMATCH {
            let h4 = hash4_mid(full, ip);
            let pos4 = table.hash4[h4];
            table.hash4[h4] = ip;
            if pos4 != usize::MAX
                && ip - pos4 <= LZ4_DISTANCE_MAX
                && full[pos4..pos4 + MINMATCH] == full[ip..ip + MINMATCH]
            {
                match_len = count_match(full, ip, pos4, match_limit);
                if match_len >= MINMATCH {
                    match_distance = ip - pos4;

                    let ip1 = ip + 1;
                    if ip < mflimit {
                        let h8_next = hash8_mid(full, ip1);
                        let pos8_next = table.hash8[h8_next];
                        let dist2 = ip1.saturating_sub(pos8_next);
                        if pos8_next != usize::MAX
                            && dist2 <= LZ4_DISTANCE_MAX
                            && full[pos8_next..pos8_next + MINMATCH] == full[ip1..ip1 + MINMATCH]
                        {
                            let len2 = count_match(full, ip1, pos8_next, match_limit);
                            if len2 > match_len {
                                table.hash8[h8_next] = ip1;
                                ip = ip1;
                                match_len = len2;
                                match_distance = dist2;
                            }
                        }
                    }
                }
            }
        }

        if match_len < MINMATCH {
            ip += 1 + ((ip - anchor) >> 9);
            continue;
        }

        while ip > anchor && ip > match_distance && full[ip - 1] == full[ip - match_distance - 1] {
            ip -= 1;
            match_len += 1;
        }

        table.add8_index(full, ip + 1, search_ip + 1);
        table.add8_index(full, ip + 2, search_ip + 2);
        table.add4_index(full, ip + 1, search_ip + 1);

        op = encode_sequence(full, dst, anchor, ip, match_len, match_distance, op)?;
        ip += match_len;
        anchor = ip;

        if ip >= 2 && ip - 2 < full.len().saturating_sub(LZ4MID_HASHSIZE) {
            if ip >= 5 {
                table.add8(full, ip - 5);
            }
            if ip >= 3 {
                table.add8(full, ip - 3);
            }
            table.add8(full, ip - 2);
            table.add4(full, ip - 2);
            table.add4(full, ip - 1);
        }
    }

    emit_last_literals_with_base(full, dst, base, anchor, op)
}

fn compress_block_hc(
    src: &[u8],
    dst: &mut [u8],
    compression_level: c_int,
    favor_dec_speed: bool,
) -> Option<usize> {
    if src.is_empty() || src.len() < MFLIMIT + 1 {
        return emit_last_literals(src, dst, 0, 0);
    }
    let level = normalize_hc_level(compression_level);
    if level <= 2 {
        return compress_block_lz4mid(src, dst);
    }
    if level >= 10 {
        return compress_block_hc_optimal(src, dst, 0, level, favor_dec_speed);
    }
    compress_block_hc_hashchain(src, dst, 0, compression_level)
}

/// Upstream `LZ4HC_compress_hashChain()` port, shared between no-dict and
/// dict compression. `base` is the offset inside `full` where the input to be
/// emitted starts; `full[..base]` is retained history (either empty or the
/// concatenated dictionary).
fn compress_block_hc_hashchain(
    full: &[u8],
    dst: &mut [u8],
    base: usize,
    compression_level: c_int,
) -> Option<usize> {
    let attempts = hc_search_attempts(compression_level);
    let mut table = HcTables::with_base(full.len(), base);
    if base > 0 {
        table.insert_until(full, base);
    }
    let wider_flags = if attempts > 128 {
        HC_FLAG_PATTERN_ANALYSIS
    } else {
        0
    };

    let mut anchor = base;
    let mut op = 0usize;
    let mflimit = full.len() - MFLIMIT;
    let match_limit = full.len() - LAST_LITERALS;
    let nomatch = HcMatch {
        start: 0,
        len: 0,
        off: 0,
    };
    let mut ip = base;

    while ip <= mflimit {
        let mut m1 = find_hc_match(full, &mut table, ip, match_limit, attempts);
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
                m2 = find_hc_wider_match(
                    full,
                    &mut table,
                    start2,
                    ip,
                    match_limit,
                    m1.len,
                    attempts,
                    wider_flags,
                );
                start2 = m2.start;
            } else {
                m2 = nomatch;
                start2 = 0;
            }

            if m2.len <= m1.len {
                op = encode_sequence(full, dst, anchor, ip, m1.len, m1.off, op)?;
                ip += m1.len;
                anchor = ip;
                break 'search2;
            }

            if start0 < ip && start2 < ip + m0.len {
                ip = start0;
                m1 = m0;
            }

            if start2 - ip < 3 {
                ip = start2;
                m1 = m2;
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
                        full,
                        &mut table,
                        start3,
                        start2,
                        match_limit,
                        m2.len,
                        attempts,
                        wider_flags,
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
                    op = encode_sequence(full, dst, anchor, ip, m1.len, m1.off, op)?;
                    ip += m1.len;
                    anchor = ip;

                    ip = start2;
                    op = encode_sequence(full, dst, anchor, ip, m2.len, m2.off, op)?;
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
                        op = encode_sequence(full, dst, anchor, ip, m1.len, m1.off, op)?;
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

                op = encode_sequence(full, dst, anchor, ip, m1.len, m1.off, op)?;
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

    emit_last_literals_with_base(full, dst, base, anchor, op)
}

fn compress_block_hc_with_dict(
    src: &[u8],
    dst: &mut [u8],
    dict: &[u8],
    compression_level: c_int,
    favor_dec_speed: bool,
) -> Option<usize> {
    if src.is_empty() {
        return emit_last_literals(src, dst, 0, 0);
    }
    let dict_keep = cmp::min(dict.len(), LZ4_DISTANCE_MAX);
    let dict = &dict[dict.len() - dict_keep..];
    if dict.is_empty() || src.len() < MFLIMIT + 1 {
        return compress_block_hc(src, dst, compression_level, favor_dec_speed);
    }

    let mut full = Vec::with_capacity(dict.len() + src.len());
    full.extend_from_slice(dict);
    full.extend_from_slice(src);
    let base = dict.len();
    let level = normalize_hc_level(compression_level);
    if level <= 2 {
        return compress_block_lz4mid_with_base(&full, dst, base);
    }
    if level >= 10 {
        return compress_block_hc_optimal(&full, dst, base, level, favor_dec_speed);
    }
    compress_block_hc_hashchain(&full, dst, base, compression_level)
}

fn compress_block_hc_optimal(
    full: &[u8],
    dst: &mut [u8],
    base: usize,
    compression_level: c_int,
    favor_dec_speed: bool,
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
    let mut table = HcTables::with_base(full.len(), base);
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
        let first_match = find_hc_longer_match(
            full,
            &mut table,
            ip,
            match_limit,
            MINMATCH - 1,
            attempts,
            favor_dec_speed,
        );
        if first_match.len == 0 {
            ip += 1;
            continue;
        }

        if first_match.len > sufficient_len {
            op = encode_sequence(full, dst, anchor, ip, first_match.len, first_match.off, op)?;
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
            let new_match = find_hc_longer_match(
                full,
                &mut table,
                cur_ptr,
                match_limit,
                min_len,
                attempts,
                favor_dec_speed,
            );
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

                let acceptable_price = opt[pos].price.saturating_sub(usize::from(favor_dec_speed));
                if pos > last_match_pos + 3 || price <= acceptable_price {
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
            op = encode_sequence(full, dst, anchor, ip, ml, offset, op)?;
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
        compress_block_hc_with_dict(
            src,
            dst,
            dict,
            prefs.compression_level,
            prefs.favor_dec_speed,
        )
    } else if prefs.compression_level > 0 {
        compress_block_hc(src, dst, prefs.compression_level, prefs.favor_dec_speed)
    } else {
        compress_block(src, dst, 1)
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
    favor_dec_speed: bool,
) -> Option<(usize, usize)> {
    if src.is_empty() {
        let written = compress_block_hc(src, dst, compression_level, favor_dec_speed)?;
        return Some((0, written));
    }

    let mut low = 0usize;
    let mut high = src.len();
    let mut best: Option<(usize, usize, Vec<u8>)> = None;
    while low <= high {
        let mid = low + (high - low) / 2;
        let mut candidate = vec![0u8; dst.len()];
        match compress_block_hc(
            &src[..mid],
            &mut candidate,
            compression_level,
            favor_dec_speed,
        ) {
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
    favor_dec_speed: bool,
) -> Option<(usize, usize)> {
    if src.is_empty() {
        let written =
            compress_block_hc_with_dict(src, dst, dict, compression_level, favor_dec_speed)?;
        return Some((0, written));
    }

    let mut low = 0usize;
    let mut high = src.len();
    let mut best: Option<(usize, usize, Vec<u8>)> = None;
    while low <= high {
        let mid = low + (high - low) / 2;
        let mut candidate = vec![0u8; dst.len()];
        match compress_block_hc_with_dict(
            &src[..mid],
            &mut candidate,
            dict,
            compression_level,
            favor_dec_speed,
        ) {
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

const HC_FLAG_PATTERN_ANALYSIS: u32 = 1 << 0;
const HC_FLAG_CHAIN_SWAP: u32 = 1 << 1;
const HC_FLAG_FAVOR_DEC_SPEED: u32 = 1 << 2;

fn find_hc_match(
    src: &[u8],
    table: &mut HcTables,
    ip: usize,
    match_limit: usize,
    max_attempts: usize,
) -> HcMatch {
    let flags = if max_attempts > 128 {
        HC_FLAG_PATTERN_ANALYSIS
    } else {
        0
    };
    find_hc_wider_match(
        src,
        table,
        ip,
        ip,
        match_limit,
        MINMATCH - 1,
        max_attempts,
        flags,
    )
}

fn find_hc_longer_match(
    src: &[u8],
    table: &mut HcTables,
    ip: usize,
    match_limit: usize,
    min_len: usize,
    max_attempts: usize,
    favor_dec_speed: bool,
) -> HcMatch {
    let mut flags = HC_FLAG_PATTERN_ANALYSIS | HC_FLAG_CHAIN_SWAP;
    if favor_dec_speed {
        flags |= HC_FLAG_FAVOR_DEC_SPEED;
    }
    let m = find_hc_wider_match(
        src,
        table,
        ip,
        ip,
        match_limit,
        min_len,
        max_attempts,
        flags,
    );
    if m.len <= min_len {
        HcMatch {
            start: ip,
            len: 0,
            off: 0,
        }
    } else {
        let len = if favor_dec_speed && m.len > 18 && m.len <= 36 {
            18
        } else {
            m.len
        };
        HcMatch { len, ..m }
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
    flags: u32,
) -> HcMatch {
    let pattern_analysis = flags & HC_FLAG_PATTERN_ANALYSIS != 0;
    let chain_swap = flags & HC_FLAG_CHAIN_SWAP != 0;
    let favor_dec_speed = flags & HC_FLAG_FAVOR_DEC_SPEED != 0;
    table.insert_until(src, ip);
    if ip + MINMATCH > match_limit {
        return HcMatch {
            start: ip,
            len: 0,
            off: 0,
        };
    }

    let ip_log = HcTables::to_log(ip);
    // `ipIndex - LZ4_DISTANCE_MAX`, clamped at `HC_OFFSET`: below `HC_OFFSET`
    // means "before the start of valid data". Matches upstream's
    // `max(lowLimit, ipIndex - LZ4_DISTANCE_MAX)`.
    let lowest_log = cmp::max(HC_OFFSET, ip_log.wrapping_sub(LZ4_DISTANCE_MAX as u32));
    let mut candidate_log = table.hash[hash4_hc(src, ip)];
    let mut attempts = max_attempts;
    let mut best = HcMatch {
        start: ip,
        len: longest,
        off: 0,
    };
    let pattern = read_u32(&src[ip..]);
    let repeated_pattern = is_repeated_pattern(pattern);
    let mut src_pattern_len = 0usize;
    let mut match_chain_pos = 0usize;
    let src_ptr = src.as_ptr();
    let src_len = src.len();
    let look_back = ip - low_limit;
    let prefix_base = table.base;
    // `HcTables::with_base` always allocates `LZ4_DISTANCE_MAX + 1` entries,
    // mirroring upstream's fixed-size `chainTable[LZ4HC_MAXD]`.
    let chain_ptr = table.chain.as_ptr();
    // Upstream loop condition is exactly this after the `+64 KiB` offset
    // trick: `while ((matchIndex >= lowestMatchIndex) && (nbAttempts > 0))`.
    // The `u16` delta chain + logical positions keep any wrap-around
    // impossible, so no extra `candidate < ip` or sentinel guards are needed.
    while candidate_log >= lowest_log && attempts > 0 {
        attempts -= 1;
        let candidate = HcTables::to_phys(candidate_log);
        let mut match_len = 0usize;
        let early_skip = favor_dec_speed && ip - candidate < 8;
        // Upstream early-exit filter: before the 4-byte pattern compare,
        // check 2 bytes at the shifted candidate's end-of-longest to skip
        // candidates that cannot improve on the current best.len.
        let mut passes_filter = true;
        if !early_skip && best.len >= 1 && candidate >= look_back {
            let ref_end = low_limit + best.len;
            let cand_end = candidate - look_back + best.len;
            if ref_end < src_len && cand_end < src_len {
                let a = read_u16_ptr(unsafe { src_ptr.add(ref_end - 1) });
                let b = read_u16_ptr(unsafe { src_ptr.add(cand_end - 1) });
                passes_filter = a == b;
            }
        }
        if !early_skip
            && passes_filter
            && read_u32_ptr(unsafe { src_ptr.add(candidate) }) == pattern
        {
            let forward =
                MINMATCH + count_match(src, ip + MINMATCH, candidate + MINMATCH, match_limit);
            let back = count_back(src, ip, candidate, low_limit);
            let len = forward + back;
            match_len = len;
            if len > best.len {
                best = HcMatch {
                    start: ip - back,
                    len,
                    off: ip - candidate,
                };
            }
        }

        if chain_swap && match_len == best.len && candidate + best.len <= ip {
            let mut distance_to_next: u32 = 1;
            let end = best.len.saturating_sub(MINMATCH - 1);
            let mut accel = 1usize << 4;
            let mut pos = 0usize;
            while pos < end {
                let probe = candidate + pos;
                // Chain stores deltas directly, mirroring upstream's
                // `DELTANEXTU16` return value. `0xFFFF` (init sentinel) is
                // strictly greater than anything useful, but it naturally
                // caps `distance_to_next`'s growth.
                let candidate_dist = unsafe { *chain_ptr.add(probe & LZ4_DISTANCE_MAX) } as u32;
                let step = accel >> 4;
                accel += 1;
                if candidate_dist > distance_to_next {
                    distance_to_next = candidate_dist;
                    match_chain_pos = pos;
                    accel = 1usize << 4;
                }
                pos += step;
            }
            if distance_to_next > 1 {
                candidate_log = candidate_log.wrapping_sub(distance_to_next);
                continue;
            }
        }

        let chain_probe = candidate + match_chain_pos;
        let dist_next_match = unsafe { *chain_ptr.add(chain_probe & LZ4_DISTANCE_MAX) } as u32;
        if pattern_analysis
            && dist_next_match == 1
            && repeated_pattern
            && candidate_log > lowest_log
            && match_chain_pos == 0
        {
            if src_pattern_len == 0 {
                src_pattern_len =
                    MINMATCH + count_pattern(src, ip + MINMATCH, match_limit, pattern);
            }
            let match_candidate = candidate - 1;
            // Upstream's `LZ4HC_protectDictEnd` guards the repeated-pattern
            // branch so a dict-area candidate that sits in the final 3 bytes
            // before the current prefix (i.e. would straddle the dict/prefix
            // boundary in ext-dict mode) is excluded. In our contiguous
            // buffer model this collapses to requiring
            // `match_candidate + 4 <= base` when the candidate is inside the
            // dict; candidates at or past `base` are always fine (upstream's
            // U32 wrap produces a value >= 3 in that case).
            let protect_dict_end = prefix_base == 0
                || match_candidate >= prefix_base
                || match_candidate + MINMATCH <= prefix_base;
            let lowest = HcTables::to_phys(lowest_log);
            if protect_dict_end
                && match_candidate >= lowest
                && match_candidate + MINMATCH <= src.len()
                && read_u32_ptr(unsafe { src.as_ptr().add(match_candidate) }) == pattern
            {
                let forward =
                    MINMATCH + count_pattern(src, match_candidate + MINMATCH, match_limit, pattern);
                // Upstream reverse-counts the pattern back to either
                // `prefixPtr` or `dictStart` depending on whether the
                // candidate is in the current prefix or external dictionary,
                // and then clamps the returned length so
                // `matchCandidateIdx - backLength >= lowestMatchIndex`. Our
                // contiguous-buffer model combines both bounds into the
                // single `0` floor for `reverseCountPattern`, so we just
                // apply the distance-based clamp afterwards.
                let back_raw = reverse_count_pattern(src, match_candidate, 0, pattern);
                let back = cmp::min(back_raw, match_candidate.saturating_sub(lowest));
                let current_segment_len = back + forward;
                let adjusted_to_segment_end =
                    current_segment_len >= src_pattern_len && forward <= src_pattern_len;
                let mut next_candidate = if adjusted_to_segment_end {
                    match_candidate + forward - src_pattern_len
                } else {
                    match_candidate.saturating_sub(back)
                };
                if next_candidate < lowest {
                    next_candidate = lowest;
                }
                if adjusted_to_segment_end {
                    if next_candidate < candidate {
                        candidate_log = HcTables::to_log(next_candidate);
                        continue;
                    }
                } else if low_limit == ip
                    && next_candidate < ip
                    && ip - next_candidate <= LZ4_DISTANCE_MAX
                {
                    let max_len = cmp::min(current_segment_len, src_pattern_len);
                    if max_len > best.len && !(favor_dec_speed && ip - next_candidate < 8) {
                        best = HcMatch {
                            start: ip,
                            len: max_len,
                            off: ip - next_candidate,
                        };
                    }
                    if next_candidate < candidate {
                        let after_pattern = table.previous(next_candidate);
                        candidate_log =
                            if after_pattern != usize::MAX && after_pattern < next_candidate {
                                HcTables::to_log(after_pattern)
                            } else {
                                HcTables::to_log(next_candidate)
                            };
                        continue;
                    }
                }

                if next_candidate < candidate {
                    candidate_log = HcTables::to_log(next_candidate);
                    continue;
                }
            }
        }

        // `candidate_log -= dist_next_match` can't wrap below `HC_OFFSET`
        // because `candidate_log >= lowest_log >= HC_OFFSET` (≥ 65536) and
        // `dist_next_match <= 65535`. If it drops below `lowest_log` the
        // outer loop condition exits on the next turn.
        candidate_log = candidate_log.wrapping_sub(dist_next_match);
    }

    best
}

fn is_repeated_pattern(pattern: u32) -> bool {
    (pattern & 0xffff) == (pattern >> 16) && (pattern & 0xff) == (pattern >> 24)
}

fn count_pattern(src: &[u8], mut pos: usize, limit: usize, pattern: u32) -> usize {
    let byte = pattern as u8;
    let start = pos;
    while pos < limit && src[pos] == byte {
        pos += 1;
    }
    pos - start
}

fn reverse_count_pattern(src: &[u8], mut pos: usize, low_limit: usize, pattern: u32) -> usize {
    let byte = pattern as u8;
    let start = pos;
    while pos > low_limit && src[pos - 1] == byte {
        pos -= 1;
    }
    start - pos
}

fn count_back(src: &[u8], ip: usize, candidate: usize, low_limit: usize) -> usize {
    let mut back = 0usize;
    let max_back = cmp::min(ip - low_limit, candidate);
    let src_ptr = src.as_ptr();
    unsafe {
        while back + 8 <= max_back {
            let a = read_u64_ptr(src_ptr.add(ip - back - 8));
            let b = read_u64_ptr(src_ptr.add(candidate - back - 8));
            let diff = a ^ b;
            if diff == 0 {
                back += 8;
            } else {
                back += diff.leading_zeros() as usize / 8;
                return back;
            }
        }
        while back < max_back && *src_ptr.add(ip - back - 1) == *src_ptr.add(candidate - back - 1) {
            back += 1;
        }
    }
    back
}

#[inline]
fn count_match(src: &[u8], mut ip: usize, mut match_pos: usize, limit: usize) -> usize {
    let start = ip;
    let src_ptr = src.as_ptr();
    unsafe {
        while ip + 8 <= limit {
            let diff = read_u64_ptr(src_ptr.add(ip)) ^ read_u64_ptr(src_ptr.add(match_pos));
            if diff != 0 {
                return ip - start + (diff.trailing_zeros() as usize / 8);
            }
            ip += 8;
            match_pos += 8;
        }
        while ip < limit && *src_ptr.add(ip) == *src_ptr.add(match_pos) {
            ip += 1;
            match_pos += 1;
        }
    }
    ip - start
}

#[inline(always)]
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

#[inline]
fn decompress_block(src: &[u8], dst: &mut [u8]) -> Option<usize> {
    let mut ip = 0usize;
    let mut op = 0usize;
    while ip < src.len() {
        let token = src[ip];
        ip += 1;

        let lit_len = read_len(src, &mut ip, (token >> 4) as usize)?;
        if ip + lit_len > src.len() || op + lit_len > dst.len() {
            return None;
        }
        unsafe {
            ptr::copy_nonoverlapping(src.as_ptr().add(ip), dst.as_mut_ptr().add(op), lit_len);
        }
        ip += lit_len;
        op += lit_len;
        if ip == src.len() {
            return Some(op);
        }
        if ip + 2 > src.len() {
            return None;
        }
        let offset = read_u16(&src[ip..]) as usize;
        ip += 2;
        if offset == 0 || offset > op {
            return None;
        }
        let match_len = read_len(src, &mut ip, (token & 0x0f) as usize)? + MINMATCH;
        if op + match_len > dst.len() {
            return None;
        }
        copy_match_no_dict(dst, &mut op, offset, match_len)?;
    }
    Some(op)
}

#[inline]
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
        let offset = read_u16(&src[ip..]) as usize;
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
        let offset = read_u16(&src[ip..]) as usize;
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

        let offset = read_u16_ptr(unsafe { src.add(ip) }) as usize;
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

#[inline]
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
        unsafe {
            ptr::copy_nonoverlapping(
                dict.as_ptr().add(dict_pos),
                dst.as_mut_ptr().add(*op),
                dict_len,
            );
        }
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
    unsafe {
        ptr::copy_nonoverlapping(dst.as_ptr().add(src), dst.as_mut_ptr().add(*op), first);
    }
    *op += first;
    len -= first;

    let mut copied = first;
    while len > 0 {
        let chunk = cmp::min(copied, len);
        let src = *op - copied;
        unsafe {
            ptr::copy_nonoverlapping(dst.as_ptr().add(src), dst.as_mut_ptr().add(*op), chunk);
        }
        *op += chunk;
        len -= chunk;
        copied += chunk;
    }
    Some(())
}

#[inline]
fn copy_match_no_dict(dst: &mut [u8], op: &mut usize, offset: usize, mut len: usize) -> Option<()> {
    if offset == 0 || offset > *op || *op + len > dst.len() {
        return None;
    }

    let first = cmp::min(offset, len);
    let src = *op - offset;
    unsafe {
        ptr::copy_nonoverlapping(dst.as_ptr().add(src), dst.as_mut_ptr().add(*op), first);
    }
    *op += first;
    len -= first;

    let mut copied = first;
    while len > 0 {
        let chunk = cmp::min(copied, len);
        let src = *op - copied;
        unsafe {
            ptr::copy_nonoverlapping(dst.as_ptr().add(src), dst.as_mut_ptr().add(*op), chunk);
        }
        *op += chunk;
        len -= chunk;
        copied += chunk;
    }
    Some(())
}

#[inline]
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

fn hash_fast(src: &[u8], pos: usize, by_u16: bool) -> usize {
    if by_u16 {
        hash4_bits(src, pos, LZ4_HASH_BITS_U16)
    } else if usize::BITS == 64 {
        let v = read_u64(&src[pos..]);
        (((v << 24).wrapping_mul(889_523_592_379)) >> (64 - LZ4_HASH_BITS)) as usize
    } else {
        hash4_bits(src, pos, LZ4_HASH_BITS)
    }
}

fn hash4_bits(src: &[u8], pos: usize, bits: usize) -> usize {
    let v = read_u32(&src[pos..]);
    ((v.wrapping_mul(2_654_435_761)) >> (32 - bits)) as usize
}

fn hash4_hc(src: &[u8], pos: usize) -> usize {
    let v = read_u32(&src[pos..]);
    ((v.wrapping_mul(2_654_435_761)) >> (32 - LZ4HC_HASH_BITS)) as usize
}

fn hash4_mid(src: &[u8], pos: usize) -> usize {
    hash4_bits(src, pos, LZ4MID_HASH_BITS)
}

fn hash8_mid(src: &[u8], pos: usize) -> usize {
    let v = read_u64(&src[pos..]);
    (((v << 8).wrapping_mul(58_295_818_150_454_627)) >> (64 - LZ4MID_HASH_BITS)) as usize
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
            dict_id: prefs.frame_info.dict_id,
            compression_level: prefs.compression_level as c_int,
            favor_dec_speed: prefs.favor_dec_speed != 0,
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
    if prefs.dict_id != 0 {
        flg |= 0x01;
    }
    out.push(flg);
    out.push(prefs.block_size_id << 4);
    if prefs.content_size != 0 {
        out.extend_from_slice(&prefs.content_size.to_le_bytes());
    }
    if prefs.dict_id != 0 {
        out.extend_from_slice(&prefs.dict_id.to_le_bytes());
    }
    let hc = (xxhash32(&out[4..], 0) >> 8) as u8;
    out.push(hc);
    out
}

fn parse_frame_header(src: &[u8]) -> Result<(FramePrefs, usize), usize> {
    if src.len() < 7 || src[..4] != LZ4F_MAGIC {
        return if src.len() >= 4 && src[..4] != LZ4F_MAGIC {
            Err(ERROR_FRAME_TYPE_UNKNOWN)
        } else {
            Err(ERROR_BAD_HEADER)
        };
    }
    let flg = src[4];
    if flg & 0xC0 != 0x40 {
        return Err(ERROR_HEADER_VERSION_WRONG);
    }
    if flg & 0x02 != 0 {
        return Err(ERROR_RESERVED_FLAG_SET);
    }
    let bd = src[5];
    let block_size_id = (bd >> 4) & 0x07;
    if bd & 0x8F != 0 {
        return Err(ERROR_RESERVED_FLAG_SET);
    }
    if !(4..=7).contains(&block_size_id) {
        return Err(ERROR_MAX_BLOCK_SIZE_INVALID);
    }
    let mut pos = 6;
    let mut content_size = 0u64;
    let mut dict_id = 0u32;
    if flg & 0x08 != 0 {
        if src.len() < pos + 8 + 1 {
            return Err(ERROR_BAD_HEADER);
        }
        content_size =
            u64::from_le_bytes(src[pos..pos + 8].try_into().map_err(|_| ERROR_BAD_HEADER)?);
        pos += 8;
    }
    if flg & 0x01 != 0 {
        if src.len() < pos + 4 + 1 {
            return Err(ERROR_BAD_HEADER);
        }
        dict_id = u32::from_le_bytes(src[pos..pos + 4].try_into().map_err(|_| ERROR_BAD_HEADER)?);
        pos += 4;
    }
    if src.len() < pos + 1 {
        return Err(ERROR_BAD_HEADER);
    }
    let expected_hc = (xxhash32(&src[4..pos], 0) >> 8) as u8;
    if src[pos] != expected_hc {
        return Err(ERROR_HEADER_CHECKSUM_INVALID);
    }
    pos += 1;
    Ok((
        FramePrefs {
            block_size_id,
            block_independent: flg & 0x20 != 0,
            block_checksum: flg & 0x10 != 0,
            content_checksum: flg & 0x04 != 0,
            content_size,
            dict_id,
            compression_level: 0,
            favor_dec_speed: false,
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
    let (prefs, header_len) = parse_frame_header(&ctx.input[ctx.pos..])?;
    ctx.pos += header_len;
    apply_frame_prefs(ctx, prefs);
    Ok(true)
}

fn apply_frame_prefs(ctx: &mut DecompressionCtx, prefs: FramePrefs) {
    ctx.parsed_header = true;
    ctx.block_checksum = prefs.block_checksum;
    ctx.content_checksum = prefs.content_checksum;
    ctx.content_size = prefs.content_size;
    ctx.content_read = 0;
    ctx.dict_id = prefs.dict_id;
    ctx.block_independent = prefs.block_independent;
    ctx.block_max = block_max_size(prefs.block_size_id);
    if !ctx.external_dictionary {
        ctx.dictionary.clear();
    }
    ctx.external_dictionary = false;
}

fn frame_info_from_decompression_ctx(ctx: &DecompressionCtx) -> LZ4FFrameInfo {
    LZ4FFrameInfo {
        block_size_id: match ctx.block_max {
            n if n == 256 * 1024 => BlockSize::Max256KB,
            n if n == 1024 * 1024 => BlockSize::Max1MB,
            n if n == 4 * 1024 * 1024 => BlockSize::Max4MB,
            _ => BlockSize::Max64KB,
        },
        block_mode: if ctx.block_independent {
            BlockMode::Independent
        } else {
            BlockMode::Linked
        },
        content_checksum_flag: if ctx.content_checksum {
            ContentChecksum::ChecksumEnabled
        } else {
            ContentChecksum::NoChecksum
        },
        frame_type: FrameType::Frame,
        content_size: ctx.content_size,
        dict_id: ctx.dict_id,
        block_checksum_flag: if ctx.block_checksum {
            BlockChecksum::BlockChecksumEnabled
        } else {
            BlockChecksum::NoBlockChecksum
        },
    }
}

fn try_decompress_frame_block_to_dst(
    ctx: &mut DecompressionCtx,
    dst: &mut [u8],
) -> Option<Result<usize, usize>> {
    if ctx.done || !ctx.parsed_header || !pending_is_empty(ctx) {
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
    if block_len > ctx.block_max {
        return Some(Err(ERROR_MAX_BLOCK_SIZE_INVALID));
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
            return Some(Err(ERROR_BLOCK_CHECKSUM_INVALID));
        }
    }

    let written = if raw {
        unsafe {
            ptr::copy_nonoverlapping(
                ctx.input.as_ptr().add(block_start),
                dst.as_mut_ptr(),
                block_len,
            );
        }
        block_len
    } else {
        let n = if ctx.block_independent && ctx.dictionary.is_empty() {
            decompress_block(&ctx.input[block_start..block_end], dst)
        } else {
            decompress_block_with_dict(&ctx.input[block_start..block_end], dst, &ctx.dictionary)
        };
        match n {
            Some(n) => n,
            None => return Some(Err(ERROR_DECOMPRESSION_FAILED)),
        }
    };

    if ctx.content_checksum {
        if raw {
            ctx.content_hasher
                .update(&ctx.input[block_start..block_end]);
        } else {
            ctx.content_hasher.update(&dst[..written]);
        }
    }
    ctx.content_read += written as u64;
    if !ctx.block_independent {
        append_hc_dictionary(&mut ctx.dictionary, &dst[..written]);
    } else {
        ctx.dictionary.clear();
    }
    ctx.pos = block_end + checksum_len;
    if ctx.content_size != 0 && ctx.content_read > ctx.content_size {
        return Some(Err(ERROR_FRAME_SIZE_WRONG));
    }
    if ctx.content_size != 0 {
        let trailer = if ctx.content_checksum { 4 } else { 0 };
        if ctx.input.len().saturating_sub(ctx.pos) >= 4 {
            let end_mark = u32::from_le_bytes(ctx.input[ctx.pos..ctx.pos + 4].try_into().unwrap());
            if end_mark != 0 {
                if ctx.content_read >= ctx.content_size {
                    return Some(Err(ERROR_FRAME_SIZE_WRONG));
                }
            } else if ctx.content_read != ctx.content_size {
                return Some(Err(ERROR_FRAME_SIZE_WRONG));
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

fn try_decompress_frame_block_slice_to_dst(
    ctx: &mut DecompressionCtx,
    src: &[u8],
    dst: &mut [u8],
) -> Option<Result<(usize, usize), usize>> {
    if src.len() < 4 {
        return None;
    }
    let block_header = u32::from_le_bytes(src[..4].try_into().unwrap());
    let raw = block_header & 0x8000_0000 != 0;
    let block_len = (block_header & 0x7FFF_FFFF) as usize;
    if block_len == 0 {
        let trailer = if ctx.content_checksum { 4 } else { 0 };
        if src.len() < 4 + trailer {
            return None;
        }
        if ctx.content_checksum {
            let stored = u32::from_le_bytes(src[4..8].try_into().unwrap());
            if stored != ctx.content_hasher.digest() {
                return Some(Err(ERROR_CHECKSUM_INVALID));
            }
        }
        if ctx.content_size != 0 && ctx.content_read != ctx.content_size {
            return Some(Err(ERROR_FRAME_SIZE_WRONG));
        }
        ctx.done = true;
        return Some(Ok((4 + trailer, 0)));
    }
    if block_len > ctx.block_max {
        return Some(Err(ERROR_MAX_BLOCK_SIZE_INVALID));
    }

    let checksum_len = if ctx.block_checksum { 4 } else { 0 };
    if src.len() < 4 + block_len + checksum_len {
        return None;
    }
    if raw {
        if dst.len() < block_len {
            return None;
        }
    } else if dst.len() < ctx.block_max {
        return None;
    }

    let block_start = 4;
    let block_end = block_start + block_len;
    if ctx.block_checksum {
        let stored =
            u32::from_le_bytes(src[block_end..block_end + checksum_len].try_into().unwrap());
        if stored != xxhash32(&src[block_start..block_end], 0) {
            return Some(Err(ERROR_BLOCK_CHECKSUM_INVALID));
        }
    }

    let written = if raw {
        unsafe {
            ptr::copy_nonoverlapping(src.as_ptr().add(block_start), dst.as_mut_ptr(), block_len);
        }
        block_len
    } else {
        let n = if ctx.block_independent && ctx.dictionary.is_empty() {
            decompress_block(&src[block_start..block_end], dst)
        } else {
            decompress_block_with_dict(&src[block_start..block_end], dst, &ctx.dictionary)
        };
        match n {
            Some(n) => n,
            None => return Some(Err(ERROR_DECOMPRESSION_FAILED)),
        }
    };

    if ctx.content_checksum {
        if raw {
            ctx.content_hasher.update(&src[block_start..block_end]);
        } else {
            ctx.content_hasher.update(&dst[..written]);
        }
    }
    ctx.content_read += written as u64;
    if !ctx.block_independent {
        append_hc_dictionary(&mut ctx.dictionary, &dst[..written]);
    } else {
        ctx.dictionary.clear();
    }
    if ctx.content_size != 0 && ctx.content_read > ctx.content_size {
        return Some(Err(ERROR_FRAME_SIZE_WRONG));
    }
    Some(Ok((4 + block_len + checksum_len, written)))
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
                return Err(ERROR_FRAME_SIZE_WRONG);
            }
            ctx.done = true;
            compact_input(ctx);
            return Ok(());
        }
        if block_len > ctx.block_max {
            return Err(ERROR_MAX_BLOCK_SIZE_INVALID);
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
                return Err(ERROR_BLOCK_CHECKSUM_INVALID);
            }
        }
        if raw {
            if ctx.content_checksum {
                ctx.content_hasher
                    .update(&ctx.input[block_start..block_end]);
            }
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
            .ok_or(ERROR_DECOMPRESSION_FAILED)?;
            if ctx.content_checksum {
                ctx.content_hasher.update(&out[..n]);
            }
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
            return Err(ERROR_FRAME_SIZE_WRONG);
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
        if ctx.pos >= ctx.input.len() {
            ctx.input.clear();
        } else {
            ctx.input.drain(..ctx.pos);
        }
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
    if src.len() < 5 || src[..4] != LZ4F_MAGIC {
        return None;
    }
    let flg = src[4];
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
        let expected = if available >= 5 {
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
        let mut v1 = self.v1;
        let mut v2 = self.v2;
        let mut v3 = self.v3;
        let mut v4 = self.v4;
        let mut p = input.as_ptr();
        let end = unsafe { p.add(input.len() & !15) };
        while p < end {
            v1 = round(v1, read_u32_ptr(p));
            v2 = round(v2, read_u32_ptr(unsafe { p.add(4) }));
            v3 = round(v3, read_u32_ptr(unsafe { p.add(8) }));
            v4 = round(v4, read_u32_ptr(unsafe { p.add(12) }));
            p = unsafe { p.add(16) };
        }
        self.v1 = v1;
        self.v2 = v2;
        self.v3 = v3;
        self.v4 = v4;
        let consumed = input.len() & !15;
        input = &input[consumed..];
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

#[inline]
fn read_u32(input: &[u8]) -> u32 {
    unsafe { ptr::read_unaligned(input.as_ptr() as *const u32).to_le() }
}

#[inline]
fn read_u16(input: &[u8]) -> u16 {
    unsafe { ptr::read_unaligned(input.as_ptr() as *const u16).to_le() }
}

#[inline]
fn read_u16_ptr(input: *const u8) -> u16 {
    unsafe { ptr::read_unaligned(input as *const u16).to_le() }
}

#[inline]
fn read_u32_ptr(input: *const u8) -> u32 {
    unsafe { ptr::read_unaligned(input as *const u32).to_le() }
}

#[inline]
fn read_u64(input: &[u8]) -> u64 {
    unsafe { ptr::read_unaligned(input.as_ptr() as *const u64).to_le() }
}

#[inline]
fn read_u64_ptr(input: *const u8) -> u64 {
    unsafe { ptr::read_unaligned(input as *const u64).to_le() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_error_code_mapping_matches_upstream_numbers() {
        unsafe {
            assert_eq!(LZ4F_isError(0), 0);
            assert_eq!(LZ4F_getErrorCode(0), 0);
            assert_eq!(
                std::ffi::CStr::from_ptr(LZ4F_getErrorName(0)).to_bytes(),
                b"Unspecified error code"
            );

            let cases = [
                (ERROR_GENERIC, 1, b"ERROR_GENERIC".as_slice()),
                (
                    ERROR_MAX_BLOCK_SIZE_INVALID,
                    2,
                    b"ERROR_maxBlockSize_invalid".as_slice(),
                ),
                (
                    ERROR_PARAMETER_INVALID,
                    4,
                    b"ERROR_parameter_invalid".as_slice(),
                ),
                (
                    ERROR_HEADER_VERSION_WRONG,
                    6,
                    b"ERROR_headerVersion_wrong".as_slice(),
                ),
                (
                    ERROR_BLOCK_CHECKSUM_INVALID,
                    7,
                    b"ERROR_blockChecksum_invalid".as_slice(),
                ),
                (
                    ERROR_RESERVED_FLAG_SET,
                    8,
                    b"ERROR_reservedFlag_set".as_slice(),
                ),
                (
                    ERROR_DST_TOO_SMALL,
                    11,
                    b"ERROR_dstMaxSize_tooSmall".as_slice(),
                ),
                (
                    ERROR_BAD_HEADER,
                    12,
                    b"ERROR_frameHeader_incomplete".as_slice(),
                ),
                (
                    ERROR_FRAME_SIZE_WRONG,
                    14,
                    b"ERROR_frameSize_wrong".as_slice(),
                ),
                (
                    ERROR_FRAME_TYPE_UNKNOWN,
                    13,
                    b"ERROR_frameType_unknown".as_slice(),
                ),
                (ERROR_SRC_PTR_WRONG, 15, b"ERROR_srcPtr_wrong".as_slice()),
                (
                    ERROR_DECOMPRESSION_FAILED,
                    16,
                    b"ERROR_decompressionFailed".as_slice(),
                ),
                (
                    ERROR_HEADER_CHECKSUM_INVALID,
                    17,
                    b"ERROR_headerChecksum_invalid".as_slice(),
                ),
                (
                    ERROR_CHECKSUM_INVALID,
                    18,
                    b"ERROR_contentChecksum_invalid".as_slice(),
                ),
                (
                    ERROR_FRAME_DECODING_ALREADY_STARTED,
                    19,
                    b"ERROR_frameDecoding_alreadyStarted".as_slice(),
                ),
                (
                    ERROR_COMPRESSION_STATE_UNINITIALIZED,
                    20,
                    b"ERROR_compressionState_uninitialized".as_slice(),
                ),
                (ERROR_PARAMETER_NULL, 21, b"ERROR_parameter_null".as_slice()),
            ];
            for (value, code, name) in cases {
                assert_eq!(LZ4F_isError(value), 1);
                assert_eq!(LZ4F_getErrorCode(value), code);
                assert_eq!(
                    std::ffi::CStr::from_ptr(LZ4F_getErrorName(value)).to_bytes(),
                    name
                );
            }
        }
    }

    #[test]
    fn frame_compression_state_errors_match_upstream() {
        unsafe {
            let mut cctx = LZ4FCompressionContext(ptr::null_mut());
            assert_eq!(LZ4F_createCompressionContext(&mut cctx, LZ4F_VERSION), 0);
            let input = b"not started";
            let mut output = vec![0u8; 128];

            assert_eq!(
                LZ4F_compressUpdate(
                    cctx,
                    output.as_mut_ptr(),
                    output.len(),
                    input.as_ptr(),
                    input.len(),
                    ptr::null(),
                ),
                ERROR_COMPRESSION_STATE_UNINITIALIZED
            );
            assert_eq!(
                LZ4F_uncompressedUpdate(
                    cctx,
                    output.as_mut_ptr() as *mut c_void,
                    output.len(),
                    input.as_ptr() as *const c_void,
                    input.len(),
                    ptr::null(),
                ),
                ERROR_COMPRESSION_STATE_UNINITIALIZED
            );
            assert_eq!(
                LZ4F_flush(cctx, output.as_mut_ptr(), output.len(), ptr::null()),
                ERROR_COMPRESSION_STATE_UNINITIALIZED
            );
            assert_eq!(
                LZ4F_compressEnd(cctx, output.as_mut_ptr(), output.len(), ptr::null()),
                ERROR_COMPRESSION_STATE_UNINITIALIZED
            );

            let header_len =
                LZ4F_compressBegin(cctx, output.as_mut_ptr(), output.len(), ptr::null());
            assert_eq!(LZ4F_isError(header_len), 0);
            assert_eq!(
                LZ4F_compressEnd(
                    cctx,
                    output.as_mut_ptr().add(header_len),
                    output.len() - header_len,
                    ptr::null(),
                ),
                4
            );
            assert_eq!(
                LZ4F_compressEnd(cctx, output.as_mut_ptr(), output.len(), ptr::null()),
                ERROR_COMPRESSION_STATE_UNINITIALIZED
            );
            LZ4F_freeCompressionContext(cctx);
        }
    }

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
    fn fast_stream_init_and_tiny_load_dict_match_upstream() {
        unsafe {
            let mut stream_storage = vec![0u8; LZ4_sizeofStreamState() as usize];
            assert!(LZ4_initStream(ptr::null_mut(), stream_storage.len()).is_null());
            assert!(LZ4_initStream(stream_storage.as_mut_ptr() as *mut c_void, 1).is_null());
            let stream = LZ4_initStream(
                stream_storage.as_mut_ptr() as *mut c_void,
                stream_storage.len(),
            );
            assert!(!stream.is_null());

            let dict = b"abcdefghijklmnop";
            for len in 0..HASH_UNIT {
                assert_eq!(
                    LZ4_loadDict(stream, dict.as_ptr() as *const c_char, len as c_int),
                    0,
                    "{len}"
                );
                assert_eq!(
                    LZ4_loadDictSlow(stream, dict.as_ptr() as *const c_char, len as c_int),
                    0,
                    "{len}"
                );
            }
            assert_eq!(
                LZ4_loadDict(stream, dict.as_ptr() as *const c_char, HASH_UNIT as c_int),
                HASH_UNIT as c_int
            );

            let mut hc_storage = vec![0u8; LZ4_sizeofStreamStateHC() as usize];
            assert!(LZ4_initStreamHC(ptr::null_mut(), hc_storage.len()).is_null());
            assert!(LZ4_initStreamHC(hc_storage.as_mut_ptr() as *mut c_void, 1).is_null());
            let hc_stream =
                LZ4_initStreamHC(hc_storage.as_mut_ptr() as *mut c_void, hc_storage.len());
            assert!(!hc_stream.is_null());
            for len in 0..HASH_UNIT {
                assert_eq!(
                    LZ4_loadDictHC(hc_stream, dict.as_ptr() as *const c_char, len as c_int),
                    len as c_int,
                    "{len}"
                );
            }
        }
    }

    #[test]
    fn hc_save_dict_treats_tiny_requested_dictionary_as_empty() {
        unsafe {
            let dict = b"abcdefghijklmnop";
            let stream = LZ4_createStreamHC();
            assert!(!stream.is_null());
            assert_eq!(
                LZ4_loadDictHC(stream, dict.as_ptr() as *const c_char, dict.len() as c_int),
                dict.len() as c_int
            );

            let mut saved = [0u8; 16];
            assert_eq!(
                LZ4_saveDictHC(stream, saved.as_mut_ptr() as *mut c_char, 3),
                0
            );
            assert_eq!(
                LZ4_saveDictHC(stream, saved.as_mut_ptr() as *mut c_char, 4),
                0
            );

            assert_eq!(
                LZ4_loadDictHC(stream, dict.as_ptr() as *const c_char, dict.len() as c_int),
                dict.len() as c_int
            );
            assert_eq!(
                LZ4_saveDictHC(stream, saved.as_mut_ptr() as *mut c_char, 4),
                4
            );
            assert_eq!(&saved[..4], &dict[dict.len() - 4..]);
            LZ4_freeStreamHC(stream);
        }
    }

    #[test]
    fn decoder_ring_buffer_size_matches_upstream_boundaries() {
        let overhead = (LZ4_DISTANCE_MAX + 1 + 14) as c_int;
        assert_eq!(LZ4_decoderRingBufferSize(-1), 0);
        assert_eq!(LZ4_decoderRingBufferSize(0), overhead + 16);
        assert_eq!(LZ4_decoderRingBufferSize(15), overhead + 16);
        assert_eq!(LZ4_decoderRingBufferSize(16), overhead + 16);
        assert_eq!(LZ4_decoderRingBufferSize(64 * 1024), overhead + 64 * 1024);
        assert_eq!(
            LZ4_decoderRingBufferSize(LZ4_MAX_INPUT_SIZE),
            overhead + LZ4_MAX_INPUT_SIZE
        );
        assert_eq!(LZ4_decoderRingBufferSize(LZ4_MAX_INPUT_SIZE + 1), 0);
    }

    #[test]
    fn fast_attach_dictionary_is_cleared_after_first_compression() {
        unsafe {
            let dict = b"abcdefghijklmnop0123456789";
            let first = b"zzzzzzzzzzzzzzzz";
            let second = b"abcdefghijklmnop0123456789";

            let dict_stream = LZ4_createStream();
            let work_stream = LZ4_createStream();
            assert!(!dict_stream.is_null());
            assert!(!work_stream.is_null());
            assert_eq!(
                LZ4_loadDict(
                    dict_stream,
                    dict.as_ptr() as *const c_char,
                    dict.len() as c_int
                ),
                dict.len() as c_int
            );
            LZ4_attach_dictionary(work_stream, dict_stream);

            let mut first_compressed = vec![0u8; LZ4_compressBound(first.len() as c_int) as usize];
            let first_len = LZ4_compress_fast_continue(
                work_stream,
                first.as_ptr() as *const c_char,
                first_compressed.as_mut_ptr() as *mut c_char,
                first.len() as c_int,
                first_compressed.len() as c_int,
                1,
            );
            assert!(first_len > 0);

            let mut second_compressed =
                vec![0u8; LZ4_compressBound(second.len() as c_int) as usize];
            let second_len = LZ4_compress_fast_continue(
                work_stream,
                second.as_ptr() as *const c_char,
                second_compressed.as_mut_ptr() as *mut c_char,
                second.len() as c_int,
                second_compressed.len() as c_int,
                1,
            );
            assert!(second_len > 0);

            let mut output = vec![0u8; second.len()];
            let output_len = LZ4_decompress_safe(
                second_compressed.as_ptr() as *const c_char,
                output.as_mut_ptr() as *mut c_char,
                second_len,
                output.len() as c_int,
            );
            assert_eq!(output_len, second.len() as c_int);
            assert_eq!(output, second);

            LZ4_freeStream(dict_stream);
            LZ4_freeStream(work_stream);
        }
    }

    #[test]
    fn hc_attach_dictionary_is_cleared_after_first_compression() {
        unsafe {
            let dict = b"abcdefghijklmnop0123456789";
            let first = b"zzzzzzzzzzzzzzzz";
            let second = b"abcdefghijklmnop0123456789";

            let dict_stream = LZ4_createStreamHC();
            let work_stream = LZ4_createStreamHC();
            assert!(!dict_stream.is_null());
            assert!(!work_stream.is_null());
            assert_eq!(
                LZ4_loadDictHC(
                    dict_stream,
                    dict.as_ptr() as *const c_char,
                    dict.len() as c_int
                ),
                dict.len() as c_int
            );
            LZ4_attach_HC_dictionary(work_stream, dict_stream);

            let mut first_compressed = vec![0u8; LZ4_compressBound(first.len() as c_int) as usize];
            let first_len = LZ4_compress_HC_continue(
                work_stream,
                first.as_ptr() as *const c_char,
                first_compressed.as_mut_ptr() as *mut c_char,
                first.len() as c_int,
                first_compressed.len() as c_int,
            );
            assert!(first_len > 0);

            let mut second_compressed =
                vec![0u8; LZ4_compressBound(second.len() as c_int) as usize];
            let second_len = LZ4_compress_HC_continue(
                work_stream,
                second.as_ptr() as *const c_char,
                second_compressed.as_mut_ptr() as *mut c_char,
                second.len() as c_int,
                second_compressed.len() as c_int,
            );
            assert!(second_len > 0);

            let mut output = vec![0u8; second.len()];
            let output_len = LZ4_decompress_safe_usingDict(
                second_compressed.as_ptr() as *const c_char,
                output.as_mut_ptr() as *mut c_char,
                second_len,
                output.len() as c_int,
                first.as_ptr() as *const c_char,
                first.len() as c_int,
            );
            assert_eq!(output_len, second.len() as c_int);
            assert_eq!(output, second);

            LZ4_freeStreamHC(dict_stream);
            LZ4_freeStreamHC(work_stream);
        }
    }

    #[test]
    fn hc_deprecated_continue_uses_stream_dictionary_and_updates_history() {
        unsafe {
            let dict = b"abcdefghijklmnop0123456789";
            let input = b"abcdefghijklmnop0123456789";
            let stream = LZ4_createStreamHC();
            assert!(!stream.is_null());
            assert_eq!(
                LZ4_loadDictHC(stream, dict.as_ptr() as *const c_char, dict.len() as c_int),
                dict.len() as c_int
            );

            let mut compressed = vec![0u8; LZ4_compressBound(input.len() as c_int) as usize];
            let compressed_len = LZ4_compressHC2_continue(
                stream as *mut c_void,
                input.as_ptr() as *const c_char,
                compressed.as_mut_ptr() as *mut c_char,
                input.len() as c_int,
                9,
            );
            assert!(compressed_len > 0);
            assert!((compressed_len as usize) < input.len());

            let mut output = vec![0u8; input.len()];
            let output_len = LZ4_decompress_safe_usingDict(
                compressed.as_ptr() as *const c_char,
                output.as_mut_ptr() as *mut c_char,
                compressed_len,
                output.len() as c_int,
                dict.as_ptr() as *const c_char,
                dict.len() as c_int,
            );
            assert_eq!(output_len, input.len() as c_int);
            assert_eq!(output, input);

            let mut saved = [0u8; 64];
            let saved_len = LZ4_saveDictHC(
                stream,
                saved.as_mut_ptr() as *mut c_char,
                saved.len() as c_int,
            );
            assert!(saved_len >= input.len() as c_int);
            let saved_len = saved_len as usize;
            assert_eq!(&saved[saved_len - input.len()..saved_len], input);
            LZ4_freeStreamHC(stream);
        }
    }

    #[test]
    fn hc_favor_decompression_speed_skips_tiny_offsets() {
        unsafe {
            let input = vec![b'a'; 4096];
            let bound = LZ4_compressBound(input.len() as c_int);
            let stream = LZ4_createStreamHC();
            assert!(!stream.is_null());
            LZ4_setCompressionLevel(stream, 10);

            let mut normal = vec![0u8; bound as usize];
            let normal_len = LZ4_compress_HC_continue(
                stream,
                input.as_ptr() as *const c_char,
                normal.as_mut_ptr() as *mut c_char,
                input.len() as c_int,
                normal.len() as c_int,
            );
            assert!(normal_len > 0);
            assert!(block_has_offset_below(&normal[..normal_len as usize], 8));

            LZ4_resetStreamHC_fast(stream, 10);
            LZ4_favorDecompressionSpeed(stream, 1);
            let mut favored = vec![0u8; bound as usize];
            let favored_len = LZ4_compress_HC_continue(
                stream,
                input.as_ptr() as *const c_char,
                favored.as_mut_ptr() as *mut c_char,
                input.len() as c_int,
                favored.len() as c_int,
            );
            LZ4_freeStreamHC(stream);

            assert!(favored_len > 0);
            assert!(favored_len >= normal_len);
            assert!(!block_has_offset_below(&favored[..favored_len as usize], 8));

            let mut output = vec![0u8; input.len()];
            let output_len = LZ4_decompress_safe(
                favored.as_ptr() as *const c_char,
                output.as_mut_ptr() as *mut c_char,
                favored_len,
                output.len() as c_int,
            );
            assert_eq!(output_len as usize, input.len());
            assert_eq!(output, input);
        }
    }

    #[test]
    fn fast_obsolete_slide_input_buffer_reports_saved_dictionary() {
        unsafe {
            let input = b"slide-input-buffer-dictionary".repeat(4);
            let stream = LZ4_createStream();
            assert!(!stream.is_null());
            assert!(LZ4_slideInputBuffer(stream as *mut c_void).is_null());

            let mut compressed = vec![0u8; LZ4_compressBound(input.len() as c_int) as usize];
            let compressed_len = LZ4_compress_fast_continue(
                stream,
                input.as_ptr() as *const c_char,
                compressed.as_mut_ptr() as *mut c_char,
                input.len() as c_int,
                compressed.len() as c_int,
                1,
            );
            assert!(compressed_len > 0);

            let ptr = LZ4_slideInputBuffer(stream as *mut c_void);
            assert!(!ptr.is_null());
            let saved = slice::from_raw_parts(ptr as *const u8, input.len());
            assert_eq!(saved, input);
            LZ4_freeStream(stream);
        }
    }

    #[test]
    fn fast_deprecated_state_wrappers_round_trip_and_reset() {
        unsafe {
            let input = b"deprecated-fast-state-wrapper-".repeat(2048);
            let bound = LZ4_compressBound(input.len() as c_int) as usize;
            let state_size = LZ4_sizeofStreamState();
            assert!(state_size > 0);
            let mut state = vec![0u8; state_size as usize];

            assert_eq!(
                LZ4_resetStreamState(
                    state.as_mut_ptr() as *mut c_void,
                    input.as_ptr() as *mut c_char,
                ),
                0
            );
            assert!(LZ4_slideInputBuffer(state.as_mut_ptr() as *mut c_void).is_null());

            let mut compressed = vec![0u8; bound];
            let compressed_len = LZ4_compress_withState(
                state.as_mut_ptr() as *mut c_void,
                input.as_ptr() as *const c_char,
                compressed.as_mut_ptr() as *mut c_char,
                input.len() as c_int,
            );
            assert!(compressed_len > 0);

            let mut output = vec![0u8; input.len()];
            let output_len = LZ4_decompress_safe(
                compressed.as_ptr() as *const c_char,
                output.as_mut_ptr() as *mut c_char,
                compressed_len,
                output.len() as c_int,
            );
            assert_eq!(output_len as usize, input.len());
            assert_eq!(output, input);

            assert_eq!(
                LZ4_resetStreamState(state.as_mut_ptr() as *mut c_void, ptr::null_mut(),),
                0
            );
            let mut limited = vec![0u8; compressed_len as usize];
            let limited_len = LZ4_compress_limitedOutput_withState(
                state.as_mut_ptr() as *mut c_void,
                input.as_ptr() as *const c_char,
                limited.as_mut_ptr() as *mut c_char,
                input.len() as c_int,
                limited.len() as c_int,
            );
            assert!(limited_len > 0);

            output.fill(0);
            let output_len = LZ4_decompress_safe(
                limited.as_ptr() as *const c_char,
                output.as_mut_ptr() as *mut c_char,
                limited_len,
                output.len() as c_int,
            );
            assert_eq!(output_len as usize, input.len());
            assert_eq!(output, input);

            let obsolete = LZ4_create(ptr::null_mut());
            assert!(!obsolete.is_null());
            assert_eq!(LZ4_freeStream(obsolete as *mut LZ4StreamEncode), 0);
        }
    }

    #[test]
    fn hc_obsolete_slide_input_buffer_clears_history() {
        unsafe {
            let dict = b"abcdefghijklmnop0123456789";
            let stream = LZ4_createStreamHC();
            assert!(!stream.is_null());
            assert!(LZ4_slideInputBufferHC(stream as *mut c_void).is_null());
            assert_eq!(
                LZ4_loadDictHC(stream, dict.as_ptr() as *const c_char, dict.len() as c_int),
                dict.len() as c_int
            );

            let ptr = LZ4_slideInputBufferHC(stream as *mut c_void);
            assert!(!ptr.is_null());

            let mut compressed = vec![0u8; LZ4_compressBound(dict.len() as c_int) as usize];
            let compressed_len = LZ4_compress_HC_continue(
                stream,
                dict.as_ptr() as *const c_char,
                compressed.as_mut_ptr() as *mut c_char,
                dict.len() as c_int,
                compressed.len() as c_int,
            );
            assert!(compressed_len > 0);

            let mut output = vec![0u8; dict.len()];
            let output_len = LZ4_decompress_safe(
                compressed.as_ptr() as *const c_char,
                output.as_mut_ptr() as *mut c_char,
                compressed_len,
                output.len() as c_int,
            );
            assert_eq!(output_len, dict.len() as c_int);
            assert_eq!(output, dict);
            LZ4_freeStreamHC(stream);
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
    fn frame_rejects_block_larger_than_declared_max_size() {
        unsafe {
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
            let mut cctx = LZ4FCompressionContext(ptr::null_mut());
            assert_eq!(LZ4F_createCompressionContext(&mut cctx, LZ4F_VERSION), 0);
            let mut header = vec![0u8; 32];
            let header_len = LZ4F_compressBegin(cctx, header.as_mut_ptr(), header.len(), &prefs);
            assert_eq!(LZ4F_isError(header_len), 0);
            header.truncate(header_len);
            LZ4F_freeCompressionContext(cctx);

            let too_large_block_len = (64 * 1024 + 1u32).to_le_bytes();
            let mut encoded = header.clone();
            encoded.extend_from_slice(&too_large_block_len);

            let mut dctx = LZ4FDecompressionContext(ptr::null_mut());
            assert_eq!(LZ4F_createDecompressionContext(&mut dctx, LZ4F_VERSION), 0);
            let mut out = [0u8; 1];
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
            assert_eq!(code, ERROR_MAX_BLOCK_SIZE_INVALID);
            LZ4F_freeDecompressionContext(dctx);

            let mut dctx = LZ4FDecompressionContext(ptr::null_mut());
            assert_eq!(LZ4F_createDecompressionContext(&mut dctx, LZ4F_VERSION), 0);
            let mut empty_dst_size = 0usize;
            let mut header_src_size = header.len();
            let hint = LZ4F_decompress(
                dctx,
                ptr::null_mut(),
                &mut empty_dst_size,
                header.as_ptr(),
                &mut header_src_size,
                ptr::null(),
            );
            assert_eq!(LZ4F_isError(hint), 0);

            let mut dst_size = out.len();
            let mut src_size = too_large_block_len.len();
            let code = LZ4F_decompress(
                dctx,
                out.as_mut_ptr(),
                &mut dst_size,
                too_large_block_len.as_ptr(),
                &mut src_size,
                ptr::null(),
            );
            assert_eq!(code, ERROR_MAX_BLOCK_SIZE_INVALID);
            LZ4F_freeDecompressionContext(dctx);
        }
    }

    #[test]
    fn frame_uncompressed_update_splits_large_raw_blocks() {
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
            let input = patterned_hc_input(150_000);
            let mut encoded = vec![0u8; LZ4F_compressBound(input.len(), &prefs) + 32];
            let mut pos = LZ4F_compressBegin(cctx, encoded.as_mut_ptr(), encoded.len(), &prefs);
            let update = LZ4F_uncompressedUpdate(
                cctx,
                encoded.as_mut_ptr().add(pos) as *mut c_void,
                encoded.len() - pos,
                input.as_ptr() as *const c_void,
                input.len(),
                ptr::null(),
            );
            assert_eq!(LZ4F_isError(update), 0);
            let first_header = u32::from_le_bytes(encoded[pos..pos + 4].try_into().unwrap());
            assert_eq!(first_header, 0x8000_0000 | 64 * 1024);
            let second_pos = pos + 4 + 64 * 1024 + 4;
            let second_header =
                u32::from_le_bytes(encoded[second_pos..second_pos + 4].try_into().unwrap());
            assert_eq!(second_header, 0x8000_0000 | 64 * 1024);
            pos += update;
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
            assert_eq!(output, input);
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
                LZ4F_headerSize(encoded.as_ptr() as *const c_void, 5),
                header_len
            );
            assert_eq!(
                LZ4F_headerSize(encoded.as_ptr() as *const c_void, 4),
                ERROR_BAD_HEADER
            );
            assert_eq!(LZ4F_headerSize(ptr::null(), 5), ERROR_SRC_PTR_WRONG);
            let bad_magic = [0u8; 5];
            assert_eq!(
                LZ4F_headerSize(bad_magic.as_ptr() as *const c_void, bad_magic.len()),
                ERROR_FRAME_TYPE_UNKNOWN
            );
            LZ4F_freeCompressionContext(cctx);

            let dict_prefs = LZ4FPreferences {
                frame_info: LZ4FFrameInfo {
                    block_size_id: BlockSize::Max64KB,
                    block_mode: BlockMode::Independent,
                    content_checksum_flag: ContentChecksum::NoChecksum,
                    frame_type: FrameType::Frame,
                    content_size: 0,
                    dict_id: 0x11,
                    block_checksum_flag: BlockChecksum::NoBlockChecksum,
                },
                compression_level: 0,
                auto_flush: 0,
                favor_dec_speed: 0,
                reserved: [0; 3],
            };
            let mut cctx = LZ4FCompressionContext(ptr::null_mut());
            assert_eq!(LZ4F_createCompressionContext(&mut cctx, LZ4F_VERSION), 0);
            let header_len =
                LZ4F_compressBegin(cctx, encoded.as_mut_ptr(), encoded.len(), &dict_prefs);
            assert_eq!(header_len, 11);
            assert_eq!(
                LZ4F_headerSize(encoded.as_ptr() as *const c_void, 5),
                header_len
            );
            LZ4F_freeCompressionContext(cctx);

            let full_prefs = LZ4FPreferences {
                frame_info: LZ4FFrameInfo {
                    block_size_id: BlockSize::Max4MB,
                    block_mode: BlockMode::Linked,
                    content_checksum_flag: ContentChecksum::ChecksumEnabled,
                    frame_type: FrameType::Frame,
                    content_size: 123,
                    dict_id: 0x22,
                    block_checksum_flag: BlockChecksum::BlockChecksumEnabled,
                },
                compression_level: 0,
                auto_flush: 0,
                favor_dec_speed: 0,
                reserved: [0; 3],
            };
            let mut cctx = LZ4FCompressionContext(ptr::null_mut());
            assert_eq!(LZ4F_createCompressionContext(&mut cctx, LZ4F_VERSION), 0);
            let header_len =
                LZ4F_compressBegin(cctx, encoded.as_mut_ptr(), encoded.len(), &full_prefs);
            assert_eq!(header_len, 19);
            assert_eq!(
                LZ4F_headerSize(encoded.as_ptr() as *const c_void, 5),
                header_len
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
    fn frame_header_preserves_dict_id_and_rejects_reserved_bits() {
        unsafe {
            let mut cctx = LZ4FCompressionContext(ptr::null_mut());
            assert_eq!(LZ4F_createCompressionContext(&mut cctx, LZ4F_VERSION), 0);
            let prefs = LZ4FPreferences {
                frame_info: LZ4FFrameInfo {
                    block_size_id: BlockSize::Max256KB,
                    block_mode: BlockMode::Independent,
                    content_checksum_flag: ContentChecksum::NoChecksum,
                    frame_type: FrameType::Frame,
                    content_size: 0,
                    dict_id: 0x1234_abcd,
                    block_checksum_flag: BlockChecksum::NoBlockChecksum,
                },
                compression_level: 0,
                auto_flush: 0,
                favor_dec_speed: 0,
                reserved: [0; 3],
            };
            let mut encoded = vec![0u8; 32];
            let header_len = LZ4F_compressBegin(cctx, encoded.as_mut_ptr(), encoded.len(), &prefs);
            assert_eq!(header_len, 11);
            assert_eq!(encoded[4] & 0x01, 0x01);

            let mut dctx = LZ4FDecompressionContext(ptr::null_mut());
            assert_eq!(LZ4F_createDecompressionContext(&mut dctx, LZ4F_VERSION), 0);
            let mut info = LZ4FFrameInfo {
                block_size_id: BlockSize::Default,
                block_mode: BlockMode::Linked,
                content_checksum_flag: ContentChecksum::ChecksumEnabled,
                frame_type: FrameType::SkippableFrame,
                content_size: 1,
                dict_id: 0,
                block_checksum_flag: BlockChecksum::BlockChecksumEnabled,
            };
            let mut src_size = header_len;
            assert_eq!(
                LZ4F_getFrameInfo(dctx, &mut info, encoded.as_ptr(), &mut src_size),
                0
            );
            assert_eq!(src_size, header_len);
            assert_eq!(info.dict_id, 0x1234_abcd);
            assert!(matches!(info.block_size_id, BlockSize::Max256KB));
            LZ4F_freeDecompressionContext(dctx);

            let mut bad_flg = encoded[..header_len].to_vec();
            bad_flg[4] |= 0x02;
            let hc = (xxhash32(&bad_flg[4..header_len - 1], 0) >> 8) as u8;
            bad_flg[header_len - 1] = hc;
            let mut bad_size = bad_flg.len();
            assert_eq!(
                LZ4F_getFrameInfo(
                    LZ4FDecompressionContext(ptr::null_mut()),
                    ptr::null_mut(),
                    ptr::null(),
                    ptr::null_mut(),
                ),
                ERROR_PARAMETER_NULL
            );
            let mut dctx_bad = LZ4FDecompressionContext(ptr::null_mut());
            assert_eq!(
                LZ4F_createDecompressionContext(&mut dctx_bad, LZ4F_VERSION),
                0
            );
            assert_eq!(
                LZ4F_getFrameInfo(dctx_bad, &mut info, bad_flg.as_ptr(), &mut bad_size),
                ERROR_RESERVED_FLAG_SET
            );

            let mut bad_bd = encoded[..header_len].to_vec();
            bad_bd[5] |= 0x01;
            let hc = (xxhash32(&bad_bd[4..header_len - 1], 0) >> 8) as u8;
            bad_bd[header_len - 1] = hc;
            bad_size = bad_bd.len();
            assert_eq!(
                LZ4F_getFrameInfo(dctx_bad, &mut info, bad_bd.as_ptr(), &mut bad_size),
                ERROR_RESERVED_FLAG_SET
            );

            let mut bad_version = encoded[..header_len].to_vec();
            bad_version[4] = (bad_version[4] & !0xC0) | 0x80;
            let hc = (xxhash32(&bad_version[4..header_len - 1], 0) >> 8) as u8;
            bad_version[header_len - 1] = hc;
            bad_size = bad_version.len();
            assert_eq!(
                LZ4F_getFrameInfo(dctx_bad, &mut info, bad_version.as_ptr(), &mut bad_size),
                ERROR_HEADER_VERSION_WRONG
            );
            let mut dst = [0u8; 1];
            let mut dst_size = dst.len();
            let mut src_size = bad_version.len();
            assert_eq!(
                LZ4F_decompress(
                    dctx_bad,
                    dst.as_mut_ptr(),
                    &mut dst_size,
                    bad_version.as_ptr(),
                    &mut src_size,
                    ptr::null(),
                ),
                ERROR_HEADER_VERSION_WRONG
            );

            LZ4F_resetDecompressionContext(dctx_bad);
            let mut bad_block_size = encoded[..header_len].to_vec();
            bad_block_size[5] = (bad_block_size[5] & 0x0f) | (3 << 4);
            let hc = (xxhash32(&bad_block_size[4..header_len - 1], 0) >> 8) as u8;
            bad_block_size[header_len - 1] = hc;
            bad_size = bad_block_size.len();
            assert_eq!(
                LZ4F_getFrameInfo(dctx_bad, &mut info, bad_block_size.as_ptr(), &mut bad_size),
                ERROR_MAX_BLOCK_SIZE_INVALID
            );
            let mut dst_size = dst.len();
            let mut src_size = bad_block_size.len();
            assert_eq!(
                LZ4F_decompress(
                    dctx_bad,
                    dst.as_mut_ptr(),
                    &mut dst_size,
                    bad_block_size.as_ptr(),
                    &mut src_size,
                    ptr::null(),
                ),
                ERROR_MAX_BLOCK_SIZE_INVALID
            );

            LZ4F_resetDecompressionContext(dctx_bad);
            let mut bad_magic = encoded[..header_len].to_vec();
            bad_magic[..4].copy_from_slice(b"bad!");
            bad_size = bad_magic.len();
            assert_eq!(
                LZ4F_getFrameInfo(dctx_bad, &mut info, bad_magic.as_ptr(), &mut bad_size),
                ERROR_FRAME_TYPE_UNKNOWN
            );
            assert_eq!(bad_size, 0);
            let mut dst_size = dst.len();
            let mut src_size = bad_magic.len();
            assert_eq!(
                LZ4F_decompress(
                    dctx_bad,
                    dst.as_mut_ptr(),
                    &mut dst_size,
                    bad_magic.as_ptr(),
                    &mut src_size,
                    ptr::null(),
                ),
                ERROR_FRAME_TYPE_UNKNOWN
            );
            LZ4F_freeDecompressionContext(dctx_bad);
            LZ4F_freeCompressionContext(cctx);
        }
    }

    #[test]
    fn frame_info_errors_zero_consumed_and_preserve_context() {
        unsafe {
            let input = b"context survives bad frame info";
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
                compression_level: 0,
                auto_flush: 0,
                favor_dec_speed: 0,
                reserved: [0; 3],
            };
            let mut encoded = vec![0u8; LZ4F_compressBound(input.len(), &prefs) + 32];
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
            let mut info = LZ4FFrameInfo {
                block_size_id: BlockSize::Default,
                block_mode: BlockMode::Linked,
                content_checksum_flag: ContentChecksum::ChecksumEnabled,
                frame_type: FrameType::SkippableFrame,
                content_size: 99,
                dict_id: 99,
                block_checksum_flag: BlockChecksum::BlockChecksumEnabled,
            };
            let mut short_size = 6usize;
            assert_eq!(
                LZ4F_getFrameInfo(dctx, &mut info, encoded.as_ptr(), &mut short_size),
                ERROR_BAD_HEADER
            );
            assert_eq!(short_size, 0);
            assert!(matches!(info.frame_type, FrameType::SkippableFrame));

            let mut consumed = encoded.len();
            assert_eq!(
                LZ4F_getFrameInfo(dctx, &mut info, encoded.as_ptr(), &mut consumed),
                0
            );
            assert_eq!(consumed, 15);

            let mut output = vec![0u8; input.len()];
            let mut src_size = encoded.len() - consumed;
            let mut dst_size = output.len();
            let code = LZ4F_decompress(
                dctx,
                output.as_mut_ptr(),
                &mut dst_size,
                encoded.as_ptr().add(consumed),
                &mut src_size,
                ptr::null(),
            );
            assert_eq!(LZ4F_isError(code), 0);
            assert_eq!(output, input);
            LZ4F_freeDecompressionContext(dctx);
        }
    }

    #[test]
    fn frame_info_rejects_context_after_partial_header_decode_started() {
        unsafe {
            let input = b"partial header get frame info";
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
                compression_level: 0,
                auto_flush: 0,
                favor_dec_speed: 0,
                reserved: [0; 3],
            };
            let mut encoded = vec![0u8; LZ4F_compressBound(input.len(), &prefs) + 32];
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
            let mut output = [0u8; 1];
            let mut src_size = 5usize;
            let mut dst_size = output.len();
            let hint = LZ4F_decompress(
                dctx,
                output.as_mut_ptr(),
                &mut dst_size,
                encoded.as_ptr(),
                &mut src_size,
                ptr::null(),
            );
            assert_eq!(LZ4F_isError(hint), 0);
            assert_eq!(src_size, 5);
            assert_eq!(dst_size, 0);

            let mut info = LZ4FFrameInfo {
                block_size_id: BlockSize::Default,
                block_mode: BlockMode::Linked,
                content_checksum_flag: ContentChecksum::ChecksumEnabled,
                frame_type: FrameType::SkippableFrame,
                content_size: 99,
                dict_id: 99,
                block_checksum_flag: BlockChecksum::BlockChecksumEnabled,
            };
            let mut consumed = encoded.len();
            assert_eq!(
                LZ4F_getFrameInfo(dctx, &mut info, encoded.as_ptr(), &mut consumed),
                ERROR_FRAME_DECODING_ALREADY_STARTED
            );
            assert_eq!(consumed, 0);
            assert!(matches!(info.frame_type, FrameType::SkippableFrame));
            LZ4F_freeDecompressionContext(dctx);
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
    fn frame_info_consumes_header_for_subsequent_decompress() {
        unsafe {
            let input = patterned_hc_input(96 * 1024);
            let prefs = LZ4FPreferences {
                frame_info: LZ4FFrameInfo {
                    block_size_id: BlockSize::Max64KB,
                    block_mode: BlockMode::Independent,
                    content_checksum_flag: ContentChecksum::ChecksumEnabled,
                    frame_type: FrameType::Frame,
                    content_size: input.len() as u64,
                    dict_id: 0xfeed_beef,
                    block_checksum_flag: BlockChecksum::NoBlockChecksum,
                },
                compression_level: 0,
                auto_flush: 0,
                favor_dec_speed: 0,
                reserved: [0; 3],
            };
            let mut encoded = vec![0u8; LZ4F_compressBound(input.len(), &prefs) + 32];
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
            let mut info = LZ4FFrameInfo {
                block_size_id: BlockSize::Default,
                block_mode: BlockMode::Linked,
                content_checksum_flag: ContentChecksum::NoChecksum,
                frame_type: FrameType::SkippableFrame,
                content_size: 1,
                dict_id: 123,
                block_checksum_flag: BlockChecksum::BlockChecksumEnabled,
            };
            let mut consumed = encoded.len();
            assert_eq!(
                LZ4F_getFrameInfo(dctx, &mut info, encoded.as_ptr(), &mut consumed),
                0
            );
            assert_eq!(consumed, 19);
            assert!(matches!(info.frame_type, FrameType::Frame));
            assert!(matches!(info.block_size_id, BlockSize::Max64KB));
            assert_eq!(info.content_size, input.len() as u64);
            assert_eq!(info.dict_id, 0xfeed_beef);

            let mut second_consumed = 123usize;
            let second_hint = LZ4F_getFrameInfo(dctx, &mut info, ptr::null(), &mut second_consumed);
            assert_eq!(LZ4F_isError(second_hint), 0);
            assert_eq!(second_consumed, 0);
            assert_eq!(second_hint, 4);
            assert_eq!(info.content_size, input.len() as u64);
            assert_eq!(info.dict_id, 0xfeed_beef);

            let mut output = vec![0u8; input.len()];
            let mut src_offset = consumed;
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
            assert_eq!(src_offset, encoded.len());
            assert_eq!(output, input);
            LZ4F_freeDecompressionContext(dctx);
        }
    }

    #[test]
    fn frame_info_reports_state_after_partial_decode_and_reset() {
        unsafe {
            let input = patterned_hc_input(80 * 1024);
            let prefs = LZ4FPreferences {
                frame_info: LZ4FFrameInfo {
                    block_size_id: BlockSize::Max64KB,
                    block_mode: BlockMode::Linked,
                    content_checksum_flag: ContentChecksum::ChecksumEnabled,
                    frame_type: FrameType::Frame,
                    content_size: input.len() as u64,
                    dict_id: 0x0102_0304,
                    block_checksum_flag: BlockChecksum::NoBlockChecksum,
                },
                compression_level: 0,
                auto_flush: 0,
                favor_dec_speed: 0,
                reserved: [0; 3],
            };
            let mut encoded = vec![0u8; LZ4F_compressBound(input.len(), &prefs) + 32];
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
            let mut info = LZ4FFrameInfo {
                block_size_id: BlockSize::Default,
                block_mode: BlockMode::Independent,
                content_checksum_flag: ContentChecksum::NoChecksum,
                frame_type: FrameType::SkippableFrame,
                content_size: 0,
                dict_id: 0,
                block_checksum_flag: BlockChecksum::NoBlockChecksum,
            };
            let mut consumed = encoded.len();
            assert_eq!(
                LZ4F_getFrameInfo(dctx, &mut info, encoded.as_ptr(), &mut consumed),
                0
            );
            assert_eq!(consumed, 19);
            assert!(matches!(info.block_mode, BlockMode::Linked));
            assert_eq!(info.content_size, input.len() as u64);
            assert_eq!(info.dict_id, 0x0102_0304);

            let options = LZ4FDecompressOptions {
                stable_dst: 1,
                reserved: [0; 3],
            };
            let mut output = [0u8; 37];
            let mut src_size = encoded.len() - consumed;
            let mut dst_size = output.len();
            let hint = LZ4F_decompress(
                dctx,
                output.as_mut_ptr(),
                &mut dst_size,
                encoded.as_ptr().add(consumed),
                &mut src_size,
                &options,
            );
            assert_eq!(LZ4F_isError(hint), 0);
            assert_eq!(dst_size, output.len());

            let mut second_consumed = usize::MAX;
            let second_hint = LZ4F_getFrameInfo(dctx, &mut info, ptr::null(), &mut second_consumed);
            assert_eq!(LZ4F_isError(second_hint), 0);
            assert_eq!(second_consumed, 0);
            assert!(matches!(info.block_mode, BlockMode::Linked));
            assert_eq!(info.content_size, input.len() as u64);
            assert_eq!(info.dict_id, 0x0102_0304);

            LZ4F_resetDecompressionContext(dctx);
            second_consumed = usize::MAX;
            assert_eq!(
                LZ4F_getFrameInfo(dctx, &mut info, ptr::null(), &mut second_consumed),
                ERROR_SRC_PTR_WRONG
            );
            assert_eq!(second_consumed, 0);
            LZ4F_freeDecompressionContext(dctx);
        }
    }

    #[test]
    fn frame_dict_id_decodes_from_upstream_generated_fixture() {
        unsafe {
            let encoded = decode_hex(
                "04224d186d401b00000000000000adfbcadeba1b0000806672616d65207769746820757073747265616d206469637420696400000000af334300",
            );
            let expected = b"frame with upstream dict id";
            let mut dctx = LZ4FDecompressionContext(ptr::null_mut());
            assert_eq!(LZ4F_createDecompressionContext(&mut dctx, LZ4F_VERSION), 0);
            let mut info = LZ4FFrameInfo {
                block_size_id: BlockSize::Default,
                block_mode: BlockMode::Linked,
                content_checksum_flag: ContentChecksum::NoChecksum,
                frame_type: FrameType::SkippableFrame,
                content_size: 0,
                dict_id: 0,
                block_checksum_flag: BlockChecksum::BlockChecksumEnabled,
            };
            let mut consumed = encoded.len();
            assert_eq!(
                LZ4F_getFrameInfo(dctx, &mut info, encoded.as_ptr(), &mut consumed),
                0
            );
            assert_eq!(consumed, 19);
            assert!(matches!(info.block_size_id, BlockSize::Max64KB));
            assert!(matches!(info.block_mode, BlockMode::Independent));
            assert!(matches!(
                info.content_checksum_flag,
                ContentChecksum::ChecksumEnabled
            ));
            assert_eq!(info.content_size, expected.len() as u64);
            assert_eq!(info.dict_id, 0xdecafbad);

            let mut output = vec![0u8; expected.len()];
            let mut src_offset = consumed;
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
            assert_eq!(src_offset, encoded.len());
            assert_eq!(dst_offset, expected.len());
            assert_eq!(output, expected);
            LZ4F_freeDecompressionContext(dctx);
        }
    }

    #[test]
    fn frame_decompress_resets_context_after_complete_frame() {
        unsafe {
            let first = b"first complete frame";
            let second = b"second complete frame";
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
            let mut first_frame = vec![0u8; LZ4F_compressBound(first.len(), &prefs) + 32];
            let first_len = LZ4F_compressFrame(
                first_frame.as_mut_ptr() as *mut c_void,
                first_frame.len(),
                first.as_ptr() as *const c_void,
                first.len(),
                &prefs,
            );
            assert_eq!(LZ4F_isError(first_len), 0);
            first_frame.truncate(first_len);

            let mut second_frame = vec![0u8; LZ4F_compressBound(second.len(), &prefs) + 32];
            let second_len = LZ4F_compressFrame(
                second_frame.as_mut_ptr() as *mut c_void,
                second_frame.len(),
                second.as_ptr() as *const c_void,
                second.len(),
                &prefs,
            );
            assert_eq!(LZ4F_isError(second_len), 0);
            second_frame.truncate(second_len);

            let mut concatenated = first_frame.clone();
            concatenated.extend_from_slice(&second_frame);

            let mut dctx = LZ4FDecompressionContext(ptr::null_mut());
            assert_eq!(LZ4F_createDecompressionContext(&mut dctx, LZ4F_VERSION), 0);
            let mut output = vec![0u8; first.len()];
            let mut src_offset = 0usize;
            let mut dst_offset = 0usize;
            loop {
                let mut src_size = first_frame.len() - src_offset;
                let mut dst_size = output.len() - dst_offset;
                let code = LZ4F_decompress(
                    dctx,
                    output[dst_offset..].as_mut_ptr(),
                    &mut dst_size,
                    concatenated.as_ptr().add(src_offset),
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
            assert_eq!(src_offset, first_frame.len());
            assert_eq!(output, first);

            let mut info = LZ4FFrameInfo {
                block_size_id: BlockSize::Default,
                block_mode: BlockMode::Linked,
                content_checksum_flag: ContentChecksum::ChecksumEnabled,
                frame_type: FrameType::SkippableFrame,
                content_size: 1,
                dict_id: 1,
                block_checksum_flag: BlockChecksum::BlockChecksumEnabled,
            };
            let mut consumed = usize::MAX;
            assert_eq!(
                LZ4F_getFrameInfo(dctx, &mut info, ptr::null(), &mut consumed),
                ERROR_SRC_PTR_WRONG
            );
            assert_eq!(consumed, 0);

            let mut output = vec![0u8; second.len()];
            src_offset = first_frame.len();
            let mut dst_offset = 0usize;
            loop {
                let mut src_size = concatenated.len() - src_offset;
                let mut dst_size = output.len() - dst_offset;
                let code = LZ4F_decompress(
                    dctx,
                    output[dst_offset..].as_mut_ptr(),
                    &mut dst_size,
                    concatenated.as_ptr().add(src_offset),
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
            assert_eq!(src_offset, concatenated.len());
            assert_eq!(output, second);
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
    fn frame_update_multiblock_dst_too_small_returns_error() {
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
            let input = vec![0x5au8; 128 * 1024];
            let mut output = vec![0u8; 16];
            assert_eq!(
                LZ4F_compressBegin(cctx, output.as_mut_ptr(), output.len(), &prefs),
                7
            );
            let code = LZ4F_compressUpdate(
                cctx,
                output.as_mut_ptr(),
                output.len(),
                input.as_ptr(),
                input.len(),
                ptr::null(),
            );
            assert_eq!(code, ERROR_DST_TOO_SMALL);
            LZ4F_freeCompressionContext(cctx);
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
            assert_eq!(code, ERROR_HEADER_CHECKSUM_INVALID);
            LZ4F_freeDecompressionContext(dctx);
        }
    }

    #[test]
    fn frame_decompress_rejects_malformed_compressed_block() {
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
            let mut encoded = vec![0u8; 32];
            let header_len = LZ4F_compressBegin(cctx, encoded.as_mut_ptr(), encoded.len(), &prefs);
            assert_eq!(LZ4F_isError(header_len), 0);
            encoded.truncate(header_len);
            LZ4F_freeCompressionContext(cctx);

            let cases = [
                ("offset past output", [0x00, 0x01, 0x00]),
                ("zero offset", [0x00, 0x00, 0x00]),
            ];
            for (case, payload) in cases {
                let mut frame = encoded.clone();
                frame.extend_from_slice(&(payload.len() as u32).to_le_bytes());
                frame.extend_from_slice(&payload);

                let mut dctx = LZ4FDecompressionContext(ptr::null_mut());
                assert_eq!(LZ4F_createDecompressionContext(&mut dctx, LZ4F_VERSION), 0);
                let mut output = vec![0u8; 64 * 1024];
                let mut src_size = frame.len();
                let mut dst_size = output.len();
                let code = LZ4F_decompress(
                    dctx,
                    output.as_mut_ptr(),
                    &mut dst_size,
                    frame.as_ptr(),
                    &mut src_size,
                    ptr::null(),
                );
                assert_eq!(code, ERROR_DECOMPRESSION_FAILED, "{case}");
                LZ4F_freeDecompressionContext(dctx);
            }
        }
    }

    #[test]
    fn frame_decompress_reports_hints_for_truncated_frame_parts() {
        unsafe {
            let no_checksum = frame_header_for_test(
                ContentChecksum::NoChecksum,
                BlockChecksum::NoBlockChecksum,
                0,
            );
            assert_incomplete_frame_hint(
                "partial block header",
                {
                    let mut frame = no_checksum.clone();
                    frame.extend_from_slice(&[0x05, 0x00]);
                    frame
                },
                2,
                0,
            );
            assert_incomplete_frame_hint(
                "partial raw block payload",
                {
                    let mut frame = no_checksum.clone();
                    frame.extend_from_slice(&(0x8000_0000u32 | 5).to_le_bytes());
                    frame.extend_from_slice(b"ab");
                    frame
                },
                3,
                0,
            );

            let with_block_checksum = frame_header_for_test(
                ContentChecksum::NoChecksum,
                BlockChecksum::BlockChecksumEnabled,
                0,
            );
            assert_incomplete_frame_hint(
                "missing block checksum",
                {
                    let mut frame = with_block_checksum;
                    frame.extend_from_slice(&(0x8000_0000u32 | 3).to_le_bytes());
                    frame.extend_from_slice(b"abc");
                    frame
                },
                4,
                0,
            );

            let with_content_checksum = frame_header_for_test(
                ContentChecksum::ChecksumEnabled,
                BlockChecksum::NoBlockChecksum,
                0,
            );
            assert_incomplete_frame_hint(
                "missing content checksum trailer",
                {
                    let mut frame = with_content_checksum;
                    frame.extend_from_slice(&(0x8000_0000u32 | 3).to_le_bytes());
                    frame.extend_from_slice(b"abc");
                    frame.extend_from_slice(&0u32.to_le_bytes());
                    frame
                },
                4,
                3,
            );
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
            assert_eq!(code, ERROR_FRAME_SIZE_WRONG);
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
                if code == ERROR_BLOCK_CHECKSUM_INVALID || code == ERROR_CHECKSUM_INVALID {
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

    unsafe fn frame_header_for_test(
        content_checksum: ContentChecksum,
        block_checksum: BlockChecksum,
        content_size: u64,
    ) -> Vec<u8> {
        let mut cctx = LZ4FCompressionContext(ptr::null_mut());
        assert_eq!(LZ4F_createCompressionContext(&mut cctx, LZ4F_VERSION), 0);
        let prefs = LZ4FPreferences {
            frame_info: LZ4FFrameInfo {
                block_size_id: BlockSize::Max64KB,
                block_mode: BlockMode::Independent,
                content_checksum_flag: content_checksum,
                frame_type: FrameType::Frame,
                content_size,
                dict_id: 0,
                block_checksum_flag: block_checksum,
            },
            compression_level: 0,
            auto_flush: 0,
            favor_dec_speed: 0,
            reserved: [0; 3],
        };
        let mut header = vec![0u8; 32];
        let header_len = LZ4F_compressBegin(cctx, header.as_mut_ptr(), header.len(), &prefs);
        assert_eq!(LZ4F_isError(header_len), 0);
        LZ4F_freeCompressionContext(cctx);
        header.truncate(header_len);
        header
    }

    unsafe fn assert_incomplete_frame_hint(
        case: &str,
        frame: Vec<u8>,
        expected_hint: usize,
        expected_output: usize,
    ) {
        let mut dctx = LZ4FDecompressionContext(ptr::null_mut());
        assert_eq!(LZ4F_createDecompressionContext(&mut dctx, LZ4F_VERSION), 0);
        let mut output = vec![0u8; 64 * 1024];
        let mut src_size = frame.len();
        let mut dst_size = output.len();
        let code = LZ4F_decompress(
            dctx,
            output.as_mut_ptr(),
            &mut dst_size,
            frame.as_ptr(),
            &mut src_size,
            ptr::null(),
        );
        assert_eq!(LZ4F_isError(code), 0, "{case}");
        assert_eq!(code, expected_hint, "{case}");
        assert_eq!(dst_size, expected_output, "{case}");
        assert_eq!(src_size, frame.len(), "{case}");
        LZ4F_freeDecompressionContext(dctx);
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
                1,
                patterned_hc_input(128),
                "ff144142434445464741426a6b6c6d6e6f70303132333435363738396162636465666768691a002b144762000d4e00503334353637",
            ),
            (
                2,
                patterned_hc_input(1024),
                "ff144142434445464741426a6b6c6d6e6f70303132333435363738396162636465666768691a002b144762000f4e00320f9c0000144662000d1a000fd00034144562000f4e00320f520100144462000d1a000f86013403e3012f43444e00320f86010003e3012d42431a000f860134144162000f4e00320f860100144762000d1a000f860134144662000f4e00320f86010003e3012d45461a000f86010450666768696a",
            ),
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
                1,
                b"the quick brown fox jumps over the lazy dog. "
                    .iter()
                    .copied()
                    .cycle()
                    .take(4096)
                    .collect::<Vec<_>>(),
                "f01074686520717569636b2062726f776e20666f78206a756d7073206f766572201f00916c617a7920646f672e0e000f2d00ffffffffffffffffffffffffffffffc6506f672e2074",
            ),
            (
                1,
                patterned_hc_input(512),
                "ff144142434445464741426a6b6c6d6e6f70303132333435363738396162636465666768691a002b144762000f4e00320f9c000000bd0001c4000fb60039086800011f010062000f1e01400168000281013f4344456c012d014e000fba010000bd0001880109d401506e6f703031",
            ),
            (
                4,
                patterned_hc_input(512),
                "ff2e4142434445464741426a6b6c6d6e6f70303132333435363738396162636465666768696a6b6c6d6e6f7030313233343536373839616263646566676869340011144762000f3400180f68001a144662000fb60039088200144562000f1e0140016800144462000f6c012d014e000fa0010003e301294344d401506e6f703031",
            ),
        ];

        for (acceleration, input, expected_hex) in cases {
            let expected = decode_hex(expected_hex);
            let mut compressed =
                vec![0u8; unsafe { LZ4_compressBound(input.len() as c_int) } as usize];
            let compressed_len = unsafe {
                LZ4_compress_fast(
                    input.as_ptr() as *const c_char,
                    compressed.as_mut_ptr() as *mut c_char,
                    input.len() as c_int,
                    compressed.len() as c_int,
                    acceleration,
                )
            };
            assert_eq!(compressed_len as usize, expected.len(), "{acceleration}");
            assert_eq!(
                &compressed[..compressed_len as usize],
                &expected,
                "{acceleration}"
            );
        }
    }

    #[test]
    fn fast_continue_matches_upstream_bytes_with_dictionary() {
        unsafe {
            let dict = b"abcdefghijklmnop0123456789abcdefghijklmnop0123456789";
            let input = b"abcdefghijklmnop0123456789ZZabcdefghijklmnop0123456789";
            for acceleration in [1, 4] {
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
                    acceleration,
                );
                LZ4_freeStream(stream);

                assert_eq!(compressed_len as usize, expected.len(), "{acceleration}");
                assert_eq!(
                    &compressed[..compressed_len as usize],
                    &expected,
                    "{acceleration}"
                );
            }
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
            let cases = [
                (9, 4504usize, 0x859b_76b8u32),
                (12, 4504usize, 0x8eb7_3b33u32),
            ];

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

    #[test]
    fn hc_large_frame_matches_upstream_hash() {
        unsafe {
            let input = patterned_hc_input(8 * 1024 * 1024);
            let prefs = hc_frame_fixture_prefs(9);
            let mut encoded = vec![0u8; LZ4F_compressFrameBound(input.len(), &prefs)];
            let encoded_len = LZ4F_compressFrame(
                encoded.as_mut_ptr() as *mut c_void,
                encoded.len(),
                input.as_ptr() as *const c_void,
                input.len(),
                &prefs,
            );

            assert_eq!(LZ4F_isError(encoded_len), 0);
            assert_eq!(encoded_len, 199_444);
            assert_eq!(xxhash32(&encoded[..encoded_len], 0), 0x6f9f_bc8e);
        }
    }

    #[test]
    fn hc_large_frame_with_cli_block_size_matches_upstream_hash() {
        unsafe {
            let input = patterned_hc_input(8 * 1024 * 1024);
            let mut prefs = hc_frame_fixture_prefs(9);
            prefs.frame_info.block_size_id = BlockSize::Max4MB;
            let mut encoded = vec![0u8; LZ4F_compressFrameBound(input.len(), &prefs)];
            let encoded_len = LZ4F_compressFrame(
                encoded.as_mut_ptr() as *mut c_void,
                encoded.len(),
                input.as_ptr() as *const c_void,
                input.len(),
                &prefs,
            );

            assert_eq!(LZ4F_isError(encoded_len), 0);
            assert_eq!(encoded_len, 35_508);
            assert_eq!(xxhash32(&encoded[..encoded_len], 0), 0x5af1_b15f);
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

    fn block_has_offset_below(block: &[u8], threshold: usize) -> bool {
        let mut pos = 0usize;
        while pos < block.len() {
            let token = block[pos];
            pos += 1;
            let mut lit_len = (token >> 4) as usize;
            if lit_len == 15 {
                loop {
                    if pos >= block.len() {
                        return false;
                    }
                    let value = block[pos] as usize;
                    pos += 1;
                    lit_len += value;
                    if value != 255 {
                        break;
                    }
                }
            }
            pos += lit_len;
            if pos >= block.len() {
                return false;
            }
            if pos + 2 > block.len() {
                return false;
            }
            let offset = u16::from_le_bytes([block[pos], block[pos + 1]]) as usize;
            if offset < threshold {
                return true;
            }
            pos += 2;
            if token & 0x0f == 15 {
                loop {
                    if pos >= block.len() {
                        return false;
                    }
                    let value = block[pos] as usize;
                    pos += 1;
                    if value != 255 {
                        break;
                    }
                }
            }
        }
        false
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
    fn hc_deprecated_state_wrappers_round_trip_and_reset() {
        let input = b"deprecated-state-hc-wrapper-".repeat(2048);
        let bound = unsafe { LZ4_compressBound(input.len() as c_int) } as usize;
        let state_size = LZ4_sizeofStreamStateHC();
        assert!(state_size > 0);
        let mut state = vec![0u8; state_size as usize];

        let reset = unsafe {
            LZ4_resetStreamStateHC(
                state.as_mut_ptr() as *mut c_void,
                input.as_ptr() as *mut c_char,
            )
        };
        assert_eq!(reset, 0);
        assert_eq!(
            unsafe { LZ4_resetStreamStateHC(ptr::null_mut(), ptr::null_mut()) },
            1
        );

        let mut compressed = vec![0u8; bound];
        let compressed_len = unsafe {
            LZ4_compressHC2_withStateHC(
                state.as_mut_ptr() as *mut c_void,
                input.as_ptr() as *const c_char,
                compressed.as_mut_ptr() as *mut c_char,
                input.len() as c_int,
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

        let mut limited = vec![0u8; compressed_len as usize];
        let limited_len = unsafe {
            LZ4_compressHC_limitedOutput_withStateHC(
                state.as_mut_ptr() as *mut c_void,
                input.as_ptr() as *const c_char,
                limited.as_mut_ptr() as *mut c_char,
                input.len() as c_int,
                limited.len() as c_int,
            )
        };
        assert!(limited_len > 0);

        output.fill(0);
        let output_len = unsafe {
            LZ4_decompress_safe(
                limited.as_ptr() as *const c_char,
                output.as_mut_ptr() as *mut c_char,
                limited_len,
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

    #[test]
    fn hc_mid_levels_continue_match_upstream_bytes_with_dictionary() {
        unsafe {
            let dict = b"abcdefghijklmnop0123456789abcdefghijklmnop0123456789";
            let input = b"abcdefghijklmnop0123456789ZZabcdefghijklmnop0123456789";
            let expected = decode_hex("0f1a00072f5a5a1c0002503536373839");

            for level in [1, 2] {
                let stream = LZ4_createStreamHC();
                assert!(!stream.is_null());
                LZ4_setCompressionLevel(stream, level);
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
                LZ4_freeStreamHC(stream);

                assert_eq!(compressed_len as usize, expected.len(), "level {level}");
                assert_eq!(
                    &compressed[..compressed_len as usize],
                    &expected,
                    "level {level}"
                );
            }
        }
    }

    /// Targeted byte-level parity check for HC pattern-analysis across the
    /// dictionary/prefix boundary. The dictionary ends in a 200-byte run of
    /// `'A'`, and the input also contains a 200-byte run of `'A'` partway
    /// through, so the HC pattern-analysis branch can extend its pattern
    /// reverse-count into the dict area only as far as upstream allows.
    #[test]
    fn hc_pattern_analysis_across_dict_boundary_matches_upstream_bytes() {
        let mut dict = vec![b'A'; 200];
        for i in 0..56 {
            dict.push(b'a' + ((i % 26) as u8));
        }
        assert_eq!(dict.len(), 256);

        let mut input = Vec::with_capacity(512);
        for i in 0..64 {
            input.push(b'b' + ((i % 13) as u8));
        }
        input.extend(std::iter::repeat_n(b'A', 200));
        for i in 0..(512 - 264) {
            input.push(b'c' + ((i % 13) as u8));
        }
        assert_eq!(input.len(), 512);

        let cases: &[(c_int, &str)] = &[
            (3, "091d000f0d00201f410100b408e0001f6f0d00d3506c6d6e6f63"),
            (4, "091d000f0d00201f410100b40924010f0d00d3506c6d6e6f63"),
            (5, "091d000f0d00201f410100b40924010f0d00d3506c6d6e6f63"),
            (6, "091d000f0d00201f410100b40924010f0d00d3506c6d6e6f63"),
            (7, "091d000f0d00201f410100b40924010f0d00d3506c6d6e6f63"),
            (8, "091d000f0d00201f410100b40924010f0d00d3506c6d6e6f63"),
            (9, "091d000f0d00200f4001b50924010f0d00d3506c6d6e6f63"),
            (10, "091d000f0d00200f4001b50924010f0d00d3506c6d6e6f63"),
            (11, "091d000f0d00200f4001b50924010f0d00d3506c6d6e6f63"),
            (12, "091d000f0d00200f4001b50924010f0d00d3506c6d6e6f63"),
        ];

        for (level, expected_hex) in cases {
            let expected = decode_hex(expected_hex);
            unsafe {
                let stream = LZ4_createStreamHC();
                assert!(!stream.is_null());
                LZ4_resetStreamHC_fast(stream, *level);
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
                LZ4_freeStreamHC(stream);

                assert_eq!(compressed_len as usize, expected.len(), "level {level}");
                assert_eq!(
                    &compressed[..compressed_len as usize],
                    &expected,
                    "level {level}"
                );
            }
        }
    }

    /// Byte-level parity check for `LZ4_compress_HC_continue()` on the
    /// optimal HC levels (10..=12) with a 256-byte loaded dictionary and a
    /// 1024-byte input. Mirrors the hash-chain version below but covers
    /// `compress_block_hc_optimal()` with a non-zero `base`.
    #[test]
    fn hc_optimal_levels_continue_match_upstream_bytes_with_dictionary() {
        let dict = patterned_hc_input(256);
        let mut input = patterned_hc_input(1024);
        for (i, byte) in input.iter_mut().enumerate() {
            *byte ^= ((i >> 7) & 1) as u8;
        }
        let cases: &[(c_int, &str)] = &[
            (10, "0f00016dff0b39386063626564676669686b6a6d6c6f6e7131303332353437361a00159e47464043424544474634000f4e00100fd00010144562010f1e014010339c00234544c4000e04010f1e012e011a00144262000e34000fd000330445021f434e001e0fd00014234043c4000f1e013d049c0005a7020fa40342356d6e6fa7030a1a000fd00037144409030f34001550676669686b"),
            (11, "0f00016dff0b39386063626564676669686b6a6d6c6f6e7131303332353437361a00159e47464043424544474634000f4e00100fd00010144562010f1e014010339c00234544c4000f0401260f1a000c144262000e34000fd000330445021f434e001e0fd00014234043c4000f1e013d049c0005a7020ea4030fee013305a7030a1a000fd00037144409030f34001550676669686b"),
            (12, "0f00016dff0b39386063626564676669686b6a6d6c6f6e7131303332353437361a00159f474640434245444746340018061a000fd00010144562010f1e014010339c00234544c4000f0401260f1a000c144262000e34000fd000330445021f434e001e0fd00014234043c4000f1e013d049c0005a7020fa40341001a0005a7030a1a000fd00037144409030f34001550676669686b"),
        ];

        for (level, expected_hex) in cases {
            let expected = decode_hex(expected_hex);
            unsafe {
                let stream = LZ4_createStreamHC();
                assert!(!stream.is_null());
                LZ4_resetStreamHC_fast(stream, *level);
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
                LZ4_freeStreamHC(stream);

                assert_eq!(compressed_len as usize, expected.len(), "level {level}");
                assert_eq!(
                    &compressed[..compressed_len as usize],
                    &expected,
                    "level {level}"
                );
            }
        }
    }

    /// Byte-level parity check for `LZ4_compress_HC_continue()` across the
    /// hash-chain HC levels (3..=9) with a 256-byte loaded dictionary and a
    /// 1024-byte input. Fixtures were produced by the upstream C library
    /// using the same `patterned_hc_input()` helper the Rust tests use.
    #[test]
    fn hc_hashchain_levels_continue_match_upstream_bytes_with_dictionary() {
        let dict = patterned_hc_input(256);
        let mut input = patterned_hc_input(1024);
        for (i, byte) in input.iter_mut().enumerate() {
            *byte ^= ((i >> 7) & 1) as u8;
        }
        let cases: &[(c_int, &str)] = &[
            (3, "0f00016dff0b39386063626564676669686b6a6d6c6f6e7131303332353437361a00159e47464043424544474634000f4e00100fd00010144562010fea00070f1a0027009c00234544c4000fb600070f1a002b144262000e34000fd000330445021f434e001e0fd0001403e3012f40431e013d049c0005a7020fb600070f1a002b05a7030a1a000fd000370445021f4734001550676669686b"),
            (4, "0f00016dff0b39386063626564676669686b6a6d6c6f6e7131303332353437361a00159e47464043424544474634000f4e00100fd00010144562010f1e014010339c00234544c4000e04010f1e012e011a00144262000e34000fd000330445021f434e001e0fd0001403e3012f40431e013d049c0005a7020ed4010fee013305a7030a1a000fd000370445021f4734001550676669686b"),
            (5, "0f00016dff0b39386063626564676669686b6a6d6c6f6e7131303332353437361a00159e47464043424544474634000f4e00100fd00010144562010f1e014010339c00234544c4000e04010f1e012e011a00144262000e34000fd000330445021f434e001e0fd0001403e3012f40431e013d049c0005a7020ed4010f0c033305a7030a1a000fd000370445021f4734001550676669686b"),
            (6, "0f00016dff0b39386063626564676669686b6a6d6c6f6e7131303332353437361a00159e47464043424544474634000f4e00100fd00010144562010f1e014010339c00234544c4000e04010f1e012e011a00144262000e34000fd000330445021f434e001e0fd0001403e3012f40431e013d049c0005a7020fa40342356d6e6fa7030a1a000fd000370445021f4734001550676669686b"),
            (7, "0f00016dff0b39386063626564676669686b6a6d6c6f6e7131303332353437361a00159e47464043424544474634000f4e00100fd00010144562010f1e014010339c00234544c4000e04010f1e012e011a00144262000e34000fd000330445021f434e001e0fd0001403e3012f40431e013d049c0005a7020fa40342356d6e6fa7030a1a000fd000370445021f4734001550676669686b"),
            (8, "0f00016dff0b39386063626564676669686b6a6d6c6f6e7131303332353437361a00159e47464043424544474634000f4e00100fd00010144562010f1e014010339c00234544c4000e04010f1e012e011a00144262000e34000fd000330445021f434e001e0fd0001403e3012f40431e013d049c0005a7020fa40342356d6e6fa7030a1a000fd000370445021f4734001550676669686b"),
            (9, "0f00016dff0b39386063626564676669686b6a6d6c6f6e7131303332353437361a00159e47464043424544474634000f4e00100fd00010144562010f1e014010339c00234544c4000e04010f1e012e011a00144262000e34000fd000330445021f434e001e0fd0001403e3012f40431e013d049c0005a7020fa40342356d6e6fa7030a1a000fd000370445021f4734001550676669686b"),
        ];

        for (level, expected_hex) in cases {
            let expected = decode_hex(expected_hex);
            unsafe {
                let stream = LZ4_createStreamHC();
                assert!(!stream.is_null());
                LZ4_resetStreamHC_fast(stream, *level);
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
                LZ4_freeStreamHC(stream);

                assert_eq!(compressed_len as usize, expected.len(), "level {level}");
                assert_eq!(
                    &compressed[..compressed_len as usize],
                    &expected,
                    "level {level}"
                );
            }
        }
    }
}
