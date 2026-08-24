//! PX4 `ULog` reader.
//!
//! Format reference: <https://docs.px4.io/main/en/dev_log/ulog_file_format.html>

use crate::{error::Error, model::FlightLog};

/// `ULog` files begin with this magic sequence.
const MAGIC: &[u8] = b"ULog\x01\x12\x35";

/// Whether `bytes` looks like a `ULog` file.
#[must_use]
pub fn is_ulog(bytes: &[u8]) -> bool {
    bytes.starts_with(MAGIC)
}

/// Parse a `ULog` file.
///
/// # Errors
/// Returns [`Error::Malformed`] only for damage the reader cannot step past.
/// Truncation is reported via [`FlightLog::is_truncated`] instead.
pub fn parse(bytes: &[u8]) -> Result<FlightLog, Error> {
    if !is_ulog(bytes) {
        return Err(Error::Malformed {
            format: "ulog",
            offset: 0,
            detail: "missing ULog magic".into(),
        });
    }
    // TODO(#1): header, definitions section, parameter messages.
    Ok(FlightLog::new("ulog"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_ulog() {
        assert!(!is_ulog(b"not a log"));
    }

    #[test]
    fn accepts_magic() {
        assert!(is_ulog(MAGIC));
    }
}
