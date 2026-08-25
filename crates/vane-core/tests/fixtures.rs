#![allow(clippy::unwrap_used, clippy::expect_used, missing_docs)]

//! Builds the synthetic `ULog` fixtures in `testdata/` and proves the parser
//! reads back exactly what was written.
//!
//! Set `VANE_REGENERATE_FIXTURES=1` to rewrite the files on disk. Without it
//! this only compares, so a normal test run never mutates the repository and
//! a fixture edited by hand fails loudly.

use std::{collections::BTreeMap, fs, path::PathBuf};
use vane_core::ParamValue;

const MAGIC: &[u8] = b"ULog\x01\x12\x35";
const VERSION: u8 = 1;
const REGENERATE: &str = "VANE_REGENERATE_FIXTURES";

const MSG_FLAG_BITS: u8 = b'B';
const MSG_FORMAT: u8 = b'F';
const MSG_INFO: u8 = b'I';
const MSG_PARAMETER: u8 = b'P';
const MSG_SUBSCRIPTION: u8 = b'A';

struct UlogWriter {
    bytes: Vec<u8>,
}

impl UlogWriter {
    fn new(start_us: u64) -> Self {
        let mut bytes = MAGIC.to_vec();
        bytes.push(VERSION);
        bytes.extend_from_slice(&start_us.to_le_bytes());

        let mut writer = Self { bytes };
        writer.message(MSG_FLAG_BITS, &[0u8; 40]);
        writer.message(
            MSG_FORMAT,
            b"sensor_combined:uint64_t timestamp;float gyro_rad[3];",
        );
        writer.message(MSG_INFO, b"\x0cchar[5] ver_swver10");
        writer
    }

    fn message(&mut self, kind: u8, payload: &[u8]) {
        let size = u16::try_from(payload.len()).expect("fixture message exceeds u16");
        self.bytes.extend_from_slice(&size.to_le_bytes());
        self.bytes.push(kind);
        self.bytes.extend_from_slice(payload);
    }

    fn parameter(&mut self, name: &str, value: &ParamValue) {
        let (type_name, raw) = match value {
            ParamValue::Int(v) => ("int32_t", v.to_le_bytes()),
            ParamValue::Float(v) => ("float", v.to_le_bytes()),
            _ => unreachable!("fixtures use only the two types ULog permits"),
        };
        let key = format!("{type_name} {name}");
        let mut payload = vec![u8::try_from(key.len()).expect("parameter key too long")];
        payload.extend_from_slice(key.as_bytes());
        payload.extend_from_slice(&raw);
        self.message(MSG_PARAMETER, &payload);
    }

    fn into_complete(mut self) -> Vec<u8> {
        self.message(MSG_SUBSCRIPTION, &[0, 1, 0]);
        self.bytes
    }

    fn into_cut_mid_parameter(mut self) -> Vec<u8> {
        let key = b"float MPC_LAND_SPEED";
        self.bytes.extend_from_slice(&30u16.to_le_bytes());
        self.bytes.push(MSG_PARAMETER);
        self.bytes
            .push(u8::try_from(key.len()).expect("parameter key too long"));
        self.bytes.extend_from_slice(&key[..8]);
        self.bytes
    }
}

fn good_parameters() -> Vec<(&'static str, ParamValue)> {
    vec![
        ("BAT1_N_CELLS", ParamValue::Int(4)),
        ("CBRK_IO_SAFETY", ParamValue::Int(22027)),
        ("COM_RC_IN_MODE", ParamValue::Int(0)),
        ("MC_PITCHRATE_D", ParamValue::Float(0.003)),
        ("MPC_TILTMAX_AIR", ParamValue::Float(45.0)),
        ("MPC_XY_P", ParamValue::Float(0.95)),
        ("SYS_AUTOSTART", ParamValue::Int(4001)),
    ]
}

fn crash_parameters() -> Vec<(&'static str, ParamValue)> {
    vec![
        ("BAT1_N_CELLS", ParamValue::Int(4)),
        ("COM_RC_IN_MODE", ParamValue::Int(0)),
        ("MC_PITCHRATE_D", ParamValue::Float(0.012)),
        ("MPC_TILTMAX_AIR", ParamValue::Float(60.0)),
        ("MPC_XY_P", ParamValue::Float(1.8)),
        ("MPC_Z_VEL_MAX_UP", ParamValue::Float(3.0)),
        ("SYS_AUTOSTART", ParamValue::Int(4001)),
    ]
}

fn build(start_us: u64, parameters: &[(&str, ParamValue)]) -> UlogWriter {
    let mut writer = UlogWriter::new(start_us);
    for (name, value) in parameters {
        writer.parameter(name, value);
    }
    writer
}

fn fixtures() -> Vec<(&'static str, Vec<u8>)> {
    vec![
        (
            "good.ulg",
            build(12_345_678, &good_parameters()).into_complete(),
        ),
        (
            "crash.ulg",
            build(98_765_432, &crash_parameters()).into_complete(),
        ),
        (
            "truncated.ulg",
            build(45_000_000, &good_parameters()).into_cut_mid_parameter(),
        ),
    ]
}

fn testdata_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../testdata")
}

fn regenerate_if_requested() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        if std::env::var(REGENERATE).is_err() {
            return;
        }
        for (name, bytes) in fixtures() {
            fs::write(testdata_dir().join(name), bytes).unwrap();
        }
    });
}

#[test]
fn committed_fixtures_match_the_generator() {
    regenerate_if_requested();

    for (name, expected) in fixtures() {
        let found = fs::read(testdata_dir().join(name)).unwrap_or_else(|error| {
            panic!("{name} is missing ({error}); run with {REGENERATE}=1 to write it")
        });
        assert_eq!(
            found, expected,
            "{name} differs from the generator; rerun with {REGENERATE}=1"
        );
    }
}

fn parsed(name: &str) -> vane_core::FlightLog {
    regenerate_if_requested();
    vane_core::open(testdata_dir().join(name)).unwrap()
}

fn expected_map(parameters: &[(&str, ParamValue)]) -> BTreeMap<String, ParamValue> {
    parameters
        .iter()
        .map(|(name, value)| ((*name).to_owned(), value.clone()))
        .collect()
}

#[test]
fn good_log_round_trips() {
    let log = parsed("good.ulg");

    assert_eq!(log.format(), "ulog");
    assert_eq!(*log.params(), expected_map(&good_parameters()));
    assert_eq!(log.started_at().unwrap().as_micros(), 12_345_678);
    assert!(!log.is_truncated());
}

#[test]
fn crash_log_round_trips() {
    let log = parsed("crash.ulg");

    assert_eq!(*log.params(), expected_map(&crash_parameters()));
    assert_eq!(log.started_at().unwrap().as_micros(), 98_765_432);
    assert!(!log.is_truncated());
}

#[test]
fn truncated_log_recovers_every_parameter_before_the_cut() {
    let log = parsed("truncated.ulg");

    assert_eq!(*log.params(), expected_map(&good_parameters()));
    assert_eq!(log.started_at().unwrap().as_micros(), 45_000_000);
    assert!(log.is_truncated());
}

#[test]
fn the_two_flights_differ_in_the_documented_ways() {
    let good = parsed("good.ulg");
    let crash = parsed("crash.ulg");

    let changed: Vec<&String> = good
        .params()
        .iter()
        .filter(|(name, value)| {
            crash
                .params()
                .get(*name)
                .is_some_and(|other| other != *value)
        })
        .map(|(name, _)| name)
        .collect();
    let removed: Vec<&String> = good
        .params()
        .keys()
        .filter(|name| !crash.params().contains_key(*name))
        .collect();
    let added: Vec<&String> = crash
        .params()
        .keys()
        .filter(|name| !good.params().contains_key(*name))
        .collect();

    assert_eq!(changed, ["MC_PITCHRATE_D", "MPC_TILTMAX_AIR", "MPC_XY_P"]);
    assert_eq!(removed, ["CBRK_IO_SAFETY"]);
    assert_eq!(added, ["MPC_Z_VEL_MAX_UP"]);
}
