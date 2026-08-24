//! The format-agnostic log model every parser targets.

use std::collections::BTreeMap;

/// A parsed flight log, normalised across source formats.
#[derive(Debug, Clone, Default)]
pub struct FlightLog {
    format: &'static str,
    params: BTreeMap<String, ParamValue>,
    truncated: bool,
}

impl FlightLog {
    /// Construct a log. Intended for use by parsers in [`crate::format`].
    #[must_use]
    pub fn new(format: &'static str) -> Self {
        Self {
            format,
            params: BTreeMap::new(),
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
