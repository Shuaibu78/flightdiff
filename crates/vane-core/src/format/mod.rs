//! Per-format parsers and magic-byte detection.
//!
//! Adding a format means adding a module here and one arm to
//! [`detect_and_parse`]. Nothing outside this module should need to change.

pub mod dataflash;
pub mod ulog;

use crate::{error::Error, model::FlightLog};
use std::{fs::File, path::Path};

/// Read enough of `path` to identify it, then hand off to the right parser.
///
/// # Errors
/// See [`Error`].
pub fn detect_and_parse(path: &Path) -> Result<FlightLog, Error> {
    let file = File::open(path).map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })?;

    // Memory-mapped so a 500 MB log costs no upfront read. The mapping is
    // read-only and dropped before this function returns.
    // SAFETY-adjacent note: memmap2's `map` is unsafe upstream; it is wrapped
    // here rather than sprinkled through the parsers.
    let bytes = map_read_only(&file, path)?;

    if ulog::is_ulog(&bytes) {
        ulog::parse(&bytes)
    } else if dataflash::is_dataflash(&bytes) {
        dataflash::parse(&bytes)
    } else {
        Err(Error::UnknownFormat { path: path.to_path_buf() })
    }
}

#[allow(unsafe_code)]
fn map_read_only(file: &File, path: &Path) -> Result<memmap2::Mmap, Error> {
    // SAFETY: the map is read-only and the file is not modified by this
    // process. A concurrent external truncation is the documented risk and is
    // accepted, matching what ripgrep and similar tools do.
    unsafe { memmap2::Mmap::map(file) }.map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })
}
