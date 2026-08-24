//! One module per subcommand. Each owns its own `Args` struct so `main.rs`
//! stays a routing table.

pub(crate) mod diff;
pub(crate) mod info;
