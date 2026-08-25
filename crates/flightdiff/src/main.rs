//! `flightdiff` command line entry point.

mod cmd;

use clap::{Parser, Subcommand};

/// Fast forensics for UAV flight logs.
#[derive(Parser)]
#[command(name = "flightdiff", version, about, long_about = None)]
struct Cli {
    /// Increase logging verbosity. Repeat for more detail.
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Summarise a log: format, duration, vehicle, parameter count.
    Info(cmd::info::Args),
    /// Show parameters that differ between two logs.
    Diff(cmd::diff::Args),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    init_tracing(cli.verbose);

    match cli.command {
        Command::Info(args) => cmd::info::run(&args),
        Command::Diff(args) => cmd::diff::run(&args),
    }
}

fn init_tracing(verbose: u8) {
    let level = match verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| level.into()),
        )
        .with_writer(std::io::stderr)
        .init();
}
