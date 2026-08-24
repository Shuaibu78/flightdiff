//! PX4 `ULog` reader.
//!
//! Format reference: <https://docs.px4.io/main/en/dev_log/ulog_file_format.html>

use crate::{error::Error, model::FlightLog, model::ParamValue};

/// `ULog` files begin with this magic sequence.
const MAGIC: &[u8] = b"ULog\x01\x12\x35";

const FILE_HEADER_LEN: usize = 16;
const MESSAGE_HEADER_LEN: usize = 3;
const PARAM_VALUE_LEN: usize = 4;
const INCOMPAT_FLAGS_RANGE: std::ops::Range<usize> = 8..16;

const MSG_FLAG_BITS: u8 = b'B';
const MSG_PARAMETER: u8 = b'P';
const MSG_SUBSCRIPTION: u8 = b'A';
const MSG_LOGGED_STRING: u8 = b'L';

const INCOMPAT_DATA_APPENDED: u64 = 1 << 0;
const SUPPORTED_INCOMPAT_FLAGS: u64 = INCOMPAT_DATA_APPENDED;

const TYPE_INT32: &str = "int32_t";
const TYPE_FLOAT: &str = "float";

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

    let mut log = FlightLog::new("ulog");
    let mut offset = FILE_HEADER_LEN;
    let mut at_first_message = true;
    let mut reached_data_section = false;

    while let Some(message) = Message::read(bytes, offset) {
        if message.kind == MSG_SUBSCRIPTION || message.kind == MSG_LOGGED_STRING {
            reached_data_section = true;
            break;
        }

        if at_first_message && message.kind == MSG_FLAG_BITS {
            reject_unsupported_flags(message.payload, offset)?;
        }
        at_first_message = false;

        if message.kind == MSG_PARAMETER {
            if let Some((name, value)) = parse_parameter(message.payload) {
                log.insert_param(name, value);
            } else {
                tracing::debug!(offset, "skipping unreadable parameter message");
            }
        }

        offset = message.end;
    }

    log.set_truncated(!reached_data_section);
    Ok(log)
}

struct Message<'a> {
    kind: u8,
    payload: &'a [u8],
    end: usize,
}

impl<'a> Message<'a> {
    fn read(bytes: &'a [u8], offset: usize) -> Option<Self> {
        let header = bytes.get(offset..offset.checked_add(MESSAGE_HEADER_LEN)?)?;
        let size = usize::from(u16::from_le_bytes([header[0], header[1]]));
        let start = offset + MESSAGE_HEADER_LEN;
        let end = start.checked_add(size)?;
        Some(Self {
            kind: header[2],
            payload: bytes.get(start..end)?,
            end,
        })
    }
}

fn reject_unsupported_flags(payload: &[u8], offset: usize) -> Result<(), Error> {
    let Some(raw) = payload
        .get(INCOMPAT_FLAGS_RANGE)
        .and_then(|slice| <[u8; 8]>::try_from(slice).ok())
    else {
        return Err(Error::Malformed {
            format: "ulog",
            offset,
            detail: "flag bits message is too short to carry incompatible flags".into(),
        });
    };

    let unsupported = u64::from_le_bytes(raw) & !SUPPORTED_INCOMPAT_FLAGS;
    if unsupported != 0 {
        return Err(Error::Malformed {
            format: "ulog",
            offset,
            detail: format!("unsupported incompatible flag bits {unsupported:#x}"),
        });
    }
    Ok(())
}

fn parse_parameter(payload: &[u8]) -> Option<(String, ParamValue)> {
    let (&key_len, rest) = payload.split_first()?;
    let key_len = usize::from(key_len);
    let key = std::str::from_utf8(rest.get(..key_len)?).ok()?;
    let raw = rest.get(key_len..)?;

    let (type_name, name) = key.split_once(' ')?;
    if name.is_empty() || raw.len() != PARAM_VALUE_LEN {
        return None;
    }
    let raw = <[u8; PARAM_VALUE_LEN]>::try_from(raw).ok()?;

    let value = match type_name {
        TYPE_INT32 => ParamValue::Int(i32::from_le_bytes(raw)),
        TYPE_FLOAT => ParamValue::Float(f32::from_le_bytes(raw)),
        _ => return None,
    };
    Some((name.to_owned(), value))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn file_header() -> Vec<u8> {
        let mut out = MAGIC.to_vec();
        out.push(1);
        out.extend_from_slice(&0u64.to_le_bytes());
        out
    }

    fn message(kind: u8, payload: &[u8]) -> Vec<u8> {
        let size = u16::try_from(payload.len()).unwrap();
        let mut out = size.to_le_bytes().to_vec();
        out.push(kind);
        out.extend_from_slice(payload);
        out
    }

    fn parameter(key: &str, value: [u8; 4]) -> Vec<u8> {
        let mut payload = vec![u8::try_from(key.len()).unwrap()];
        payload.extend_from_slice(key.as_bytes());
        payload.extend_from_slice(&value);
        message(MSG_PARAMETER, &payload)
    }

    fn flag_bits(incompat: u64) -> Vec<u8> {
        let mut payload = vec![0u8; 40];
        payload[INCOMPAT_FLAGS_RANGE].copy_from_slice(&incompat.to_le_bytes());
        message(MSG_FLAG_BITS, &payload)
    }

    fn data_section() -> Vec<u8> {
        message(MSG_SUBSCRIPTION, &[0, 0, 0])
    }

    #[test]
    fn rejects_non_ulog() {
        assert!(!is_ulog(b"not a log"));
    }

    #[test]
    fn accepts_magic() {
        assert!(is_ulog(MAGIC));
    }

    #[test]
    fn reads_int_and_float_parameters() {
        let mut bytes = file_header();
        bytes.extend(parameter("int32_t SYS_AUTOSTART", 4001i32.to_le_bytes()));
        bytes.extend(parameter("float MC_ROLL_P", 6.5f32.to_le_bytes()));
        bytes.extend(data_section());

        let log = parse(&bytes).unwrap();

        assert_eq!(log.params().len(), 2);
        assert_eq!(log.params()["SYS_AUTOSTART"], ParamValue::Int(4001));
        assert_eq!(log.params()["MC_ROLL_P"], ParamValue::Float(6.5));
        assert!(!log.is_truncated());
    }

    #[test]
    fn stops_at_data_section() {
        let mut bytes = file_header();
        bytes.extend(parameter("int32_t BEFORE", 1i32.to_le_bytes()));
        bytes.extend(data_section());
        bytes.extend(parameter("int32_t AFTER", 2i32.to_le_bytes()));

        let log = parse(&bytes).unwrap();

        assert!(log.params().contains_key("BEFORE"));
        assert!(!log.params().contains_key("AFTER"));
    }

    #[test]
    fn stops_at_logged_string() {
        let mut bytes = file_header();
        bytes.extend(parameter("int32_t BEFORE", 1i32.to_le_bytes()));
        bytes.extend(message(MSG_LOGGED_STRING, &[0; 4]));
        bytes.extend(parameter("int32_t AFTER", 2i32.to_le_bytes()));

        let log = parse(&bytes).unwrap();

        assert!(log.params().contains_key("BEFORE"));
        assert!(!log.params().contains_key("AFTER"));
    }

    #[test]
    fn skips_definition_messages_it_does_not_read() {
        let mut bytes = file_header();
        bytes.extend(message(b'F', b"sensor_combined:uint64_t timestamp;"));
        bytes.extend(message(b'I', b"\x09ver_sw_abc"));
        bytes.extend(message(b'Q', b"\x0dint32_t A_DEF\x00\x00\x00\x00"));
        bytes.extend(parameter("int32_t AFTER_SKIPS", 7i32.to_le_bytes()));
        bytes.extend(data_section());

        let log = parse(&bytes).unwrap();

        assert_eq!(log.params().len(), 1);
        assert_eq!(log.params()["AFTER_SKIPS"], ParamValue::Int(7));
    }

    #[test]
    fn recovers_parameters_from_a_log_cut_mid_message() {
        let mut bytes = file_header();
        bytes.extend(parameter("int32_t RECOVERED", 9i32.to_le_bytes()));
        bytes.extend_from_slice(&64u16.to_le_bytes());
        bytes.push(MSG_PARAMETER);
        bytes.extend_from_slice(b"\x0aint32_t X");

        let log = parse(&bytes).unwrap();

        assert_eq!(log.params()["RECOVERED"], ParamValue::Int(9));
        assert!(log.is_truncated());
    }

    #[test]
    fn header_only_log_is_truncated() {
        let log = parse(&file_header()).unwrap();

        assert!(log.params().is_empty());
        assert!(log.is_truncated());
    }

    #[test]
    fn log_cut_inside_the_file_header_is_truncated() {
        let log = parse(&file_header()[..10]).unwrap();

        assert!(log.params().is_empty());
        assert!(log.is_truncated());
    }

    #[test]
    fn skips_parameters_of_unsupported_type() {
        let mut bytes = file_header();
        bytes.extend(parameter("double PRECISE", 0i32.to_le_bytes()));
        bytes.extend(parameter("int32_t KEPT", 3i32.to_le_bytes()));
        bytes.extend(data_section());

        let log = parse(&bytes).unwrap();

        assert_eq!(log.params().len(), 1);
        assert!(log.params().contains_key("KEPT"));
    }

    #[test]
    fn skips_parameter_whose_key_length_overruns_its_payload() {
        let mut bytes = file_header();
        bytes.extend(message(MSG_PARAMETER, b"\xfeint32_t A"));
        bytes.extend(parameter("int32_t KEPT", 3i32.to_le_bytes()));
        bytes.extend(data_section());

        let log = parse(&bytes).unwrap();

        assert_eq!(log.params().len(), 1);
        assert!(log.params().contains_key("KEPT"));
    }

    #[test]
    fn accepts_the_data_appended_flag() {
        let mut bytes = file_header();
        bytes.extend(flag_bits(INCOMPAT_DATA_APPENDED));
        bytes.extend(parameter("int32_t KEPT", 1i32.to_le_bytes()));
        bytes.extend(data_section());

        let log = parse(&bytes).unwrap();

        assert!(log.params().contains_key("KEPT"));
    }

    #[test]
    fn refuses_a_log_with_unsupported_incompatible_flags() {
        let mut bytes = file_header();
        bytes.extend(flag_bits(1 << 7));
        bytes.extend(parameter("int32_t KEPT", 1i32.to_le_bytes()));
        bytes.extend(data_section());

        let error = parse(&bytes).unwrap_err();

        assert!(matches!(error, Error::Malformed { format: "ulog", .. }));
    }

    #[test]
    fn ignores_flag_bits_that_are_not_the_first_message() {
        let mut bytes = file_header();
        bytes.extend(parameter("int32_t KEPT", 1i32.to_le_bytes()));
        bytes.extend(flag_bits(1 << 7));
        bytes.extend(data_section());

        let log = parse(&bytes).unwrap();

        assert!(log.params().contains_key("KEPT"));
    }

    #[test]
    fn zero_sized_messages_do_not_stall_the_reader() {
        let mut bytes = file_header();
        bytes.extend(message(b'I', &[]));
        bytes.extend(message(b'I', &[]));
        bytes.extend(parameter("int32_t KEPT", 1i32.to_le_bytes()));
        bytes.extend(data_section());

        let log = parse(&bytes).unwrap();

        assert!(log.params().contains_key("KEPT"));
    }
}
