//! One module per subcommand. Each owns its own `Args` struct so `main.rs`
//! stays a routing table.

pub mod diff;
pub mod info;
