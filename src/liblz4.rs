//! Re-exports of the low-level [`crate::sys`] surface along with thin Rust
//! error helpers used by the higher-level wrappers in this crate.

use std::ffi::CStr;
use std::fmt::Display;
use std::fmt::Formatter;
use std::io::Error;
use std::str;

pub use crate::sys::*;

/// Wraps a textual LZ4 error description.
///
/// Produced by [`check_error`] from frame-API error codes (the upstream
/// `LZ4F_getErrorName` value of an `LZ4FErrorCode`). It implements
/// [`std::error::Error`] and [`Display`] so it can be embedded in an
/// [`std::io::Error`].
#[derive(Debug)]
pub struct LZ4Error(String);

impl Display for LZ4Error {
    fn fmt(&self, f: &mut Formatter) -> Result<(), ::std::fmt::Error> {
        write!(f, "LZ4 error: {}", &self.0)
    }
}

impl ::std::error::Error for LZ4Error {
    fn description(&self) -> &str {
        &self.0
    }

    fn cause(&self) -> Option<&dyn ::std::error::Error> {
        None
    }
}

/// Converts an LZ4 frame-API return code into a `Result`.
///
/// LZ4 frame functions return a `size_t` that is either a useful size or, if
/// `LZ4F_isError` reports true, an encoded error code. This helper turns the
/// latter into an [`std::io::Error`] whose inner cause is an [`LZ4Error`]
/// carrying the upstream `LZ4F_getErrorName` string. On success the original
/// code is returned as a `usize`.
pub fn check_error(code: LZ4FErrorCode) -> Result<usize, Error> {
    unsafe {
        if LZ4F_isError(code) != 0 {
            let error_name = LZ4F_getErrorName(code);
            return Err(Error::other(LZ4Error(
                str::from_utf8(CStr::from_ptr(error_name).to_bytes())
                    .unwrap()
                    .to_string(),
            )));
        }
    }
    Ok(code)
}

/// Returns the LZ4 library version as an integer (`MAJOR*100*100 + MINOR*100 + RELEASE`),
/// matching upstream `LZ4_versionNumber`.
pub fn version() -> i32 {
    unsafe { LZ4_versionNumber() }
}

#[test]
fn test_version_number() {
    version();
}
