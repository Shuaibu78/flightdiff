//! `vane diff` — parameter differences between two logs.
//!
//! This is the first shipped feature. "What changed between the flight that
//! worked and the flight that crashed" is the question it answers.

use std::path::PathBuf;

/// Arguments for `vane diff`.
#[derive(clap::Args)]
pub struct Args {
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
pub fn run(args: &Args) -> anyhow::Result<()> {
    let before = vane_core::open(&args.before)?;
    let after = vane_core::open(&args.after)?;

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
