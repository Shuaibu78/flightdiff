# Architecture

A short map of the codebase, kept current. If a change makes this file wrong,
the change should update it.

## Crate layout

```
crates/
  vane-core/   parsing and the log model. No I/O policy, no printing.
  vane-cli/    argument parsing and output formatting. No parsing logic.
```

The split exists so other projects can depend on `vane-core` alone. That means
`vane-core` must never print, never call `std::process::exit`, and never
assume a terminal.

## Data flow

```
path -> mmap -> magic byte detection -> format parser -> FlightLog -> command
```

`FlightLog` is the narrow waist. Adding a format means producing a `FlightLog`;
adding a command means consuming one. Neither side knows about the other.

## Why memory mapping

Logs run to hundreds of megabytes and most commands touch a small part of one:
`diff` reads only the parameter section. Mapping defers the read to the pages
actually touched. The single `unsafe` block is confined to
`format/mod.rs::map_read_only` so the rest of the workspace can keep
`unsafe_code = "deny"`.

## Planned

- Columnar storage for time series, so scanning one field across a million
  samples does not pull in the others.
- A rule engine (`vane check`) encoding known failure signatures: vibration
  thresholds, EKF innovation spikes, battery sag, motor output divergence.
- Recovery of records from truncated and partially overwritten logs.
