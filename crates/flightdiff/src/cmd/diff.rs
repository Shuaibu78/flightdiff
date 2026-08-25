//! `flightdiff diff` — parameter differences between two logs.
//!
//! This is the first shipped feature. "What changed between the flight that
//! worked and the flight that crashed" is the question it answers.

use flightdiff_core::FlightLog;
use std::path::{Path, PathBuf};

/// Arguments for `flightdiff diff`.
#[derive(clap::Args)]
pub(crate) struct Args {
    /// Baseline log, usually the flight that behaved.
    pub before: PathBuf,
    /// Comparison log, usually the flight that did not.
    pub after: PathBuf,

    /// Print every parameter, not only the ones that differ.
    #[arg(long)]
    pub all: bool,
}

/// Run the subcommand.
///
/// # Errors
/// Propagates any read or parse failure on either input.
pub(crate) fn run(args: &Args) -> anyhow::Result<()> {
    let before = flightdiff_core::open(&args.before)?;
    let after = flightdiff_core::open(&args.after)?;

    note_truncation(&args.before, &before);
    note_truncation(&args.after, &after);

    let mut keys: Vec<&String> = before.params().keys().collect();
    keys.extend(after.params().keys());
    keys.sort_unstable();
    keys.dedup();

    let mut changed = 0usize;
    for key in keys {
        let lhs = before.params().get(key);
        let rhs = after.params().get(key);
        if lhs == rhs && !args.all {
            continue;
        }
        changed += 1;
        match (lhs, rhs) {
            (Some(a), Some(b)) => println!("~ {key}: {a} -> {b}"),
            (Some(a), None) => println!("- {key}: {a}"),
            (None, Some(b)) => println!("+ {key}: {b}"),
            (None, None) => unreachable!("key came from one of the two maps"),
        }
    }

    if changed == 0 {
        println!("no parameter differences");
    }
    Ok(())
}

fn note_truncation(path: &Path, log: &FlightLog) {
    if log.is_truncated() {
        println!(
            "! {} was cut short; only the parameters recovered before the cut were compared",
            path.display()
        );
    }
}
