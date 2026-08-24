//! `vane info` — summarise a single log.

use std::path::PathBuf;

/// Arguments for `vane info`.
#[derive(clap::Args)]
pub(crate) struct Args {
    /// Log file to summarise.
    pub log: PathBuf,
}

/// Run the subcommand.
///
/// # Errors
/// Propagates any read or parse failure.
pub(crate) fn run(args: &Args) -> anyhow::Result<()> {
    let log = vane_core::open(&args.log)?;
    println!("format:     {}", log.format());
    println!("parameters: {}", log.params().len());
    if log.is_truncated() {
        println!("note:       log is truncated; recovered what was readable");
    }
    Ok(())
}
