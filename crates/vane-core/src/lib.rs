//! Format-agnostic reading of UAV flight logs.
//!
//! The crate is deliberately usable on its own: other tools should be able to
//! depend on `vane-core` for parsing without pulling in the CLI.
//!
//! ```no_run
//! # fn main() -> Result<(), vane_core::Error> {
//! let log = vane_core::open("flight.ulg")?;
//! println!("{} parameters", log.params().len());
//! # Ok(())
//! # }
//! ```

pub mod error;
pub mod format;
pub mod model;

pub use error::Error;
pub use model::{FlightLog, ParamValue};

use std::path::Path;

/// Open a flight log, detecting the format from its magic bytes.
///
/// # Errors
/// Returns [`Error::Io`] if the file cannot be read and
/// [`Error::UnknownFormat`] if no registered parser recognises it.
pub fn open(path: impl AsRef<Path>) -> Result<FlightLog, Error> {
    format::detect_and_parse(path.as_ref())
}
