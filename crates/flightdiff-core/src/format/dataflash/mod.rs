//! `ArduPilot` `DataFlash` (`.bin`) reader.
//!
//! Format reference: <https://ardupilot.org/dev/docs/common-logs.html>

use crate::{error::Error, model::FlightLog, model::ParamValue};
use flightlog_core::ReadOptions;
use std::collections::HashSet;

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

    // FMT and PARM are always interpreted; an empty type set skips building
    // tables for everything else.
    let opts = ReadOptions {
        types: Some(HashSet::new()),
        ..ReadOptions::default()
    };
    let parsed = flightlog_core::dataflash::read(bytes, &opts);

    let mut log = FlightLog::new("dataflash");
    for (name, value) in parsed.params {
        #[allow(clippy::cast_possible_truncation)] // PARM.Value is an f32 on disk.
        log.insert_param(name, ParamValue::Float(value as f32));
    }
    log.set_truncated(parsed.integrity.truncated_tail_bytes > 0);
    Ok(log)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_dataflash() {
        assert!(!is_dataflash(b"xx"));
    }

    fn padded<const N: usize>(text: &str) -> [u8; N] {
        let mut out = [0u8; N];
        out[..text.len()].copy_from_slice(text.as_bytes());
        out
    }

    /// FMT for PARM (type 64, `QNf`: `TimeUS,Name,Value`) then one PARM per entry.
    fn log_with_params(params: &[(&str, f32)]) -> Vec<u8> {
        let mut bytes = vec![0xA3, 0x95, 0x80, 64, 31];
        bytes.extend_from_slice(&padded::<4>("PARM"));
        bytes.extend_from_slice(&padded::<16>("QNf"));
        bytes.extend_from_slice(&padded::<64>("TimeUS,Name,Value"));
        for (name, value) in params {
            bytes.extend_from_slice(&[0xA3, 0x95, 64]);
            bytes.extend_from_slice(&0u64.to_le_bytes());
            bytes.extend_from_slice(&padded::<16>(name));
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes
    }

    #[test]
    fn reads_parameters() {
        let log = parse(&log_with_params(&[
            ("ATC_RAT_RLL_P", 0.135),
            ("FRAME_CLASS", 1.0),
        ]))
        .unwrap();
        assert_eq!(log.params().len(), 2);
        assert_eq!(log.params()["ATC_RAT_RLL_P"], ParamValue::Float(0.135));
        assert_eq!(log.params()["FRAME_CLASS"], ParamValue::Float(1.0));
        assert!(!log.is_truncated());
    }

    #[test]
    fn keeps_parameters_from_a_cut_log() {
        let mut bytes = log_with_params(&[("FRAME_CLASS", 1.0), ("FRAME_TYPE", 3.0)]);
        bytes.truncate(bytes.len() - 5);
        let log = parse(&bytes).unwrap();
        assert_eq!(log.params()["FRAME_CLASS"], ParamValue::Float(1.0));
        assert!(!log.params().contains_key("FRAME_TYPE"));
        assert!(log.is_truncated());
    }
}
