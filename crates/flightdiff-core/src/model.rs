//! The format-agnostic log model every parser targets.

use std::{collections::BTreeMap, time::Duration};

/// A parsed flight log, normalised across source formats.
#[derive(Debug, Clone, Default)]
pub struct FlightLog {
    format: &'static str,
    params: BTreeMap<String, ParamValue>,
    started_at: Option<Duration>,
    truncated: bool,
}

impl FlightLog {
    /// Construct a log. Intended for use by parsers in [`crate::format`].
    #[must_use]
    pub fn new(format: &'static str) -> Self {
        Self {
            format,
            params: BTreeMap::new(),
            started_at: None,
            truncated: false,
        }
    }

    /// Name of the source format, e.g. `"ulog"`.
    #[must_use]
    pub fn format(&self) -> &'static str {
        self.format
    }

    /// Vehicle parameters recorded in the log header.
    #[must_use]
    pub fn params(&self) -> &BTreeMap<String, ParamValue> {
        &self.params
    }

    /// Insert a parameter. Intended for use by parsers.
    pub fn insert_param(&mut self, key: impl Into<String>, value: ParamValue) {
        self.params.insert(key.into(), value);
    }

    /// Time the log started, as recorded in its header.
    ///
    /// This is the logging start offset the format carries, not a wall-clock
    /// date, and not the length of the flight. Reporting how long a flight
    /// lasted needs the last timestamp in the data section, which this reader
    /// does not yet visit.
    #[must_use]
    pub fn started_at(&self) -> Option<Duration> {
        self.started_at
    }

    /// Record the logging start time. Intended for use by parsers.
    pub fn set_started_at(&mut self, started_at: Duration) {
        self.started_at = Some(started_at);
    }

    /// Whether parsing stopped early because the file was cut short.
    ///
    /// A truncated log is still returned with whatever was recovered; callers
    /// should surface this to the user rather than treat it as a failure.
    #[must_use]
    pub fn is_truncated(&self) -> bool {
        self.truncated
    }

    /// Mark the log as truncated. Intended for use by parsers.
    pub fn set_truncated(&mut self, truncated: bool) {
        self.truncated = truncated;
    }
}

/// A single vehicle parameter value.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum ParamValue {
    /// 32-bit signed integer parameter.
    Int(i32),
    /// 32-bit float parameter.
    Float(f32),
}

impl std::fmt::Display for ParamValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(v) => write!(f, "{v}"),
            Self::Float(v) => write!(f, "{v}"),
        }
    }
}
