//! Error type for the crate.

use std::path::PathBuf;

/// Errors produced while reading a flight log.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The underlying file could not be read.
    #[error("reading {path}")]
    Io {
        /// File that failed to open.
        path: PathBuf,
        /// Underlying cause.
        #[source]
        source: std::io::Error,
    },

    /// No registered parser recognised the file's magic bytes.
    #[error("unrecognised log format: {path}")]
    UnknownFormat {
        /// File that could not be identified.
        path: PathBuf,
    },

    /// The file was recognised but is structurally invalid.
    ///
    /// Truncated logs are common in practice (a crash cuts the write short),
    /// so parsers should prefer returning partial data over this variant.
    #[error("malformed {format} log at byte {offset}: {detail}")]
    Malformed {
        /// Format the parser believed it was reading.
        format: &'static str,
        /// Byte offset at which parsing failed.
        offset: usize,
        /// Human-readable description.
        detail: String,
    },
}
