//! `ArduPilot` `DataFlash` (`.bin`) reader.
//!
//! Format reference: <https://ardupilot.org/dev/docs/common-logs.html>

use crate::{error::Error, model::FlightLog};

/// Every `DataFlash` message begins with this two-byte header.
const HEADER: [u8; 2] = [0xA3, 0x95];

/// Whether `bytes` looks like a `DataFlash` log.
#[must_use]
pub fn is_dataflash(bytes: &[u8]) -> bool {
    bytes.starts_with(&HEADER)
}

/// Parse a `DataFlash` log.
///
/// # Errors
/// See [`Error`].
pub fn parse(bytes: &[u8]) -> Result<FlightLog, Error> {
    if !is_dataflash(bytes) {
        return Err(Error::Malformed {
            format: "dataflash",
            offset: 0,
            detail: "missing message header".into(),
        });
    }
    // TODO(#2): FMT self-describing message table, then PARM extraction.
    Ok(FlightLog::new("dataflash"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_dataflash() {
        assert!(!is_dataflash(b"xx"));
    }
}
