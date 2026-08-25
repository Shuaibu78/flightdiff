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

    let bytes = map_read_only(&file, path)?;

    if ulog::is_ulog(&bytes) {
        ulog::parse(&bytes)
    } else if dataflash::is_dataflash(&bytes) {
        dataflash::parse(&bytes)
    } else {
        Err(Error::UnknownFormat {
            path: path.to_path_buf(),
        })
    }
}

#[allow(unsafe_code)]
fn map_read_only(file: &File, path: &Path) -> Result<memmap2::Mmap, Error> {
    unsafe { memmap2::Mmap::map(file) }.map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })
}
