//! `ArduPilot` `DataFlash` (`.bin`) reader.
//!
//! Format reference: <https://ardupilot.org/dev/docs/common-logs.html>

use crate::{error::Error, model::FlightLog, model::ParamValue};
use std::collections::HashMap;

/// Every `DataFlash` message begins with this two-byte header.
const HEADER: [u8; 2] = [0xA3, 0x95];

const PACKET_HEADER_LEN: usize = 3;
const FORMAT_MSG_ID: u8 = 128;
const FORMAT_MSG_LEN: usize = 89;

const FORMAT_NAME_RANGE: std::ops::Range<usize> = 2..6;
const FORMAT_SPEC_RANGE: std::ops::Range<usize> = 6..22;
const FORMAT_LABELS_RANGE: std::ops::Range<usize> = 22..86;

const PARAM_MSG_NAME: &str = "PARM";
const PARAM_NAME_LABEL: &str = "Name";
const PARAM_VALUE_LABEL: &str = "Value";
const PARAM_NAME_SPEC: u8 = b'N';
const PARAM_VALUE_SPEC: u8 = b'f';
const PARAM_VALUE_LEN: usize = 4;

/// Whether `bytes` looks like a `DataFlash` log.
#[must_use]
pub fn is_dataflash(bytes: &[u8]) -> bool {
    bytes.starts_with(&HEADER)
}

/// Parse a `DataFlash` log.
///
/// # Errors
/// Returns [`Error::Malformed`] only for damage the reader cannot step past.
/// Truncation is reported via [`FlightLog::is_truncated`] instead.
pub fn parse(bytes: &[u8]) -> Result<FlightLog, Error> {
    if !is_dataflash(bytes) {
        return Err(Error::Malformed {
            format: "dataflash",
            offset: 0,
            detail: "missing message header".into(),
        });
    }

    let mut log = FlightLog::new("dataflash");
    let mut lengths: HashMap<u8, usize> = HashMap::new();
    let mut params: Option<(u8, ParamLayout)> = None;
    let mut offset = 0;
    let mut truncated = false;

    while offset < bytes.len() {
        let Some(start) = next_header(bytes, offset) else {
            truncated = true;
            break;
        };
        if start != offset {
            tracing::debug!(from = offset, to = start, "resynchronised after damage");
        }

        let Some(&msg_id) = bytes.get(start + 2) else {
            truncated = true;
            break;
        };

        let Some(length) = message_length(msg_id, &lengths) else {
            offset = start + HEADER.len();
            continue;
        };

        let Some(payload) = bytes.get(start + PACKET_HEADER_LEN..start + length) else {
            truncated = true;
            break;
        };

        if msg_id == FORMAT_MSG_ID {
            register_format(payload, &mut lengths, &mut params);
        } else if let Some((_, layout)) = params.as_ref().filter(|(id, _)| *id == msg_id) {
            if let Some((name, value)) = layout.read(payload) {
                log.insert_param(name, ParamValue::Float(value));
            } else {
                tracing::debug!(offset = start, "skipping unreadable PARM message");
            }
        }

        offset = start + length;
    }

    log.set_truncated(truncated);
    Ok(log)
}

struct ParamLayout {
    name: std::ops::Range<usize>,
    value: usize,
}

impl ParamLayout {
    fn read(&self, payload: &[u8]) -> Option<(String, f32)> {
        let name = fixed_str(payload.get(self.name.clone())?)?;
        if name.is_empty() {
            return None;
        }
        let raw = payload.get(self.value..self.value.checked_add(PARAM_VALUE_LEN)?)?;
        let raw = <[u8; PARAM_VALUE_LEN]>::try_from(raw).ok()?;
        Some((name.to_owned(), f32::from_le_bytes(raw)))
    }
}

fn next_header(bytes: &[u8], from: usize) -> Option<usize> {
    bytes
        .get(from..)?
        .windows(HEADER.len())
        .position(|window| window == HEADER)
        .map(|found| from + found)
}

fn message_length(msg_id: u8, lengths: &HashMap<u8, usize>) -> Option<usize> {
    let length = if msg_id == FORMAT_MSG_ID {
        FORMAT_MSG_LEN
    } else {
        *lengths.get(&msg_id)?
    };
    (length > PACKET_HEADER_LEN).then_some(length)
}

fn register_format(
    payload: &[u8],
    lengths: &mut HashMap<u8, usize>,
    params: &mut Option<(u8, ParamLayout)>,
) {
    let Some(described) = read_format(payload) else {
        tracing::debug!("skipping unreadable FMT message");
        return;
    };

    lengths.insert(described.msg_id, described.length);

    if described.name == PARAM_MSG_NAME {
        if let Some(layout) = param_layout(&described.spec, &described.labels) {
            *params = Some((described.msg_id, layout));
        } else {
            tracing::debug!(spec = described.spec, "PARM format is not readable");
        }
    }
}

struct DescribedMessage {
    msg_id: u8,
    length: usize,
    name: String,
    spec: String,
    labels: String,
}

fn read_format(payload: &[u8]) -> Option<DescribedMessage> {
    Some(DescribedMessage {
        msg_id: *payload.first()?,
        length: usize::from(*payload.get(1)?),
        name: fixed_str(payload.get(FORMAT_NAME_RANGE)?)?.to_owned(),
        spec: fixed_str(payload.get(FORMAT_SPEC_RANGE)?)?.to_owned(),
        labels: fixed_str(payload.get(FORMAT_LABELS_RANGE)?)?.to_owned(),
    })
}

fn param_layout(spec: &str, labels: &str) -> Option<ParamLayout> {
    let mut fields = Vec::new();
    let mut at = 0usize;
    for code in spec.bytes() {
        let size = field_size(code)?;
        fields.push((code, at, size));
        at = at.checked_add(size)?;
    }

    let mut name = None;
    let mut value = None;
    for (label, &(code, at, size)) in labels.split(',').zip(fields.iter()) {
        match label {
            PARAM_NAME_LABEL if code == PARAM_NAME_SPEC => name = Some(at..at + size),
            PARAM_VALUE_LABEL if code == PARAM_VALUE_SPEC => value = Some(at),
            _ => {}
        }
    }

    Some(ParamLayout {
        name: name?,
        value: value?,
    })
}

fn field_size(code: u8) -> Option<usize> {
    Some(match code {
        b'b' | b'B' | b'M' => 1,
        b'h' | b'H' | b'c' | b'C' => 2,
        b'i' | b'I' | b'f' | b'e' | b'E' | b'L' | b'n' => 4,
        b'd' | b'q' | b'Q' => 8,
        b'N' => 16,
        b'a' | b'Z' => 64,
        _ => return None,
    })
}

fn fixed_str(bytes: &[u8]) -> Option<&str> {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    std::str::from_utf8(bytes.get(..end)?).ok()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn packet(msg_id: u8, payload: &[u8]) -> Vec<u8> {
        let mut out = HEADER.to_vec();
        out.push(msg_id);
        out.extend_from_slice(payload);
        out
    }

    fn padded(text: &str, width: usize) -> Vec<u8> {
        let mut out = text.as_bytes().to_vec();
        out.resize(width, 0);
        out
    }

    fn format_message(msg_id: u8, length: u8, name: &str, spec: &str, labels: &str) -> Vec<u8> {
        let mut payload = vec![msg_id, length];
        payload.extend(padded(name, 4));
        payload.extend(padded(spec, 16));
        payload.extend(padded(labels, 64));
        packet(FORMAT_MSG_ID, &payload)
    }

    fn parm_format(msg_id: u8) -> Vec<u8> {
        format_message(msg_id, 35, "PARM", "QNff", "TimeUS,Name,Value,Default")
    }

    fn parm(msg_id: u8, name: &str, value: f32) -> Vec<u8> {
        let mut payload = 0u64.to_le_bytes().to_vec();
        payload.extend(padded(name, 16));
        payload.extend_from_slice(&value.to_le_bytes());
        payload.extend_from_slice(&0f32.to_le_bytes());
        packet(msg_id, &payload)
    }

    #[test]
    fn rejects_non_dataflash() {
        assert!(!is_dataflash(b"xx"));
    }

    #[test]
    fn reads_parameters_from_parm_messages() {
        let mut bytes = parm_format(64);
        bytes.extend(parm(64, "ATC_RAT_RLL_P", 0.135));
        bytes.extend(parm(64, "INS_GYRO_FILTER", 20.0));

        let log = parse(&bytes).unwrap();

        assert_eq!(log.params().len(), 2);
        assert_eq!(log.params()["ATC_RAT_RLL_P"], ParamValue::Float(0.135));
        assert_eq!(log.params()["INS_GYRO_FILTER"], ParamValue::Float(20.0));
        assert!(!log.is_truncated());
    }

    #[test]
    fn reads_older_logs_that_have_no_default_column() {
        let mut bytes = format_message(64, 31, "PARM", "QNf", "TimeUS,Name,Value");
        let mut payload = 0u64.to_le_bytes().to_vec();
        payload.extend(padded("RTL_ALT", 16));
        payload.extend_from_slice(&15.0f32.to_le_bytes());
        bytes.extend(packet(64, &payload));

        let log = parse(&bytes).unwrap();

        assert_eq!(log.params()["RTL_ALT"], ParamValue::Float(15.0));
    }

    #[test]
    fn ignores_messages_that_are_not_parameters() {
        let mut bytes = format_message(10, 19, "IMU", "QffI", "TimeUS,GyrX,GyrY,Id");
        bytes.extend(parm_format(64));
        bytes.extend(packet(10, &[0; 16]));
        bytes.extend(parm(64, "KEPT", 1.0));

        let log = parse(&bytes).unwrap();

        assert_eq!(log.params().len(), 1);
        assert!(log.params().contains_key("KEPT"));
    }

    #[test]
    fn skips_messages_that_have_no_format() {
        let mut bytes = parm_format(64);
        bytes.extend(packet(200, &[0; 8]));
        bytes.extend(parm(64, "KEPT", 2.0));

        let log = parse(&bytes).unwrap();

        assert_eq!(log.params()["KEPT"], ParamValue::Float(2.0));
    }

    #[test]
    fn resynchronises_after_corrupt_bytes() {
        let mut bytes = parm_format(64);
        bytes.extend(parm(64, "BEFORE", 1.0));
        bytes.extend_from_slice(b"\x00\xff garbage in the middle \x00\xff");
        bytes.extend(parm(64, "AFTER", 2.0));

        let log = parse(&bytes).unwrap();

        assert_eq!(log.params()["BEFORE"], ParamValue::Float(1.0));
        assert_eq!(log.params()["AFTER"], ParamValue::Float(2.0));
    }

    #[test]
    fn recovers_parameters_from_a_log_cut_mid_message() {
        let mut bytes = parm_format(64);
        bytes.extend(parm(64, "RECOVERED", 3.5));
        let cut = parm(64, "LOST", 4.0);
        bytes.extend_from_slice(&cut[..10]);

        let log = parse(&bytes).unwrap();

        assert_eq!(log.params()["RECOVERED"], ParamValue::Float(3.5));
        assert!(!log.params().contains_key("LOST"));
        assert!(log.is_truncated());
    }

    #[test]
    fn a_log_of_only_format_messages_yields_no_parameters() {
        let log = parse(&parm_format(64)).unwrap();

        assert!(log.params().is_empty());
        assert!(!log.is_truncated());
    }

    #[test]
    fn skips_a_format_carrying_an_unknown_field_code() {
        let mut bytes = format_message(64, 35, "PARM", "QN?f", "TimeUS,Name,Pad,Value");
        bytes.extend(parm(64, "UNREADABLE", 1.0));

        let log = parse(&bytes).unwrap();

        assert!(log.params().is_empty());
    }

    #[test]
    fn skips_a_parm_format_whose_value_field_is_not_a_float() {
        let mut bytes = format_message(64, 35, "PARM", "QNi", "TimeUS,Name,Value");
        bytes.extend(parm(64, "UNREADABLE", 1.0));

        let log = parse(&bytes).unwrap();

        assert!(log.params().is_empty());
    }

    #[test]
    fn a_format_shorter_than_its_header_does_not_stall_the_reader() {
        let mut bytes = format_message(64, 2, "PARM", "QNff", "TimeUS,Name,Value,Default");
        bytes.extend(parm_format(65));
        bytes.extend(parm(65, "KEPT", 9.0));

        let log = parse(&bytes).unwrap();

        assert_eq!(log.params()["KEPT"], ParamValue::Float(9.0));
    }

    #[test]
    fn later_definitions_of_the_same_parameter_win() {
        let mut bytes = parm_format(64);
        bytes.extend(parm(64, "CHANGED", 1.0));
        bytes.extend(parm(64, "CHANGED", 7.0));

        let log = parse(&bytes).unwrap();

        assert_eq!(log.params()["CHANGED"], ParamValue::Float(7.0));
    }
}
