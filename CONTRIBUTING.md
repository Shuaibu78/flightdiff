# Contributing

Thanks for taking the time.

## Getting set up

```sh
git clone https://github.com/Shuaibu78/flightdiff
cd flightdiff
cargo check --workspace --all-targets
cargo test --workspace
```

## Before opening a pull request

```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

CI runs exactly these three, so a clean local run means a clean CI run.

## Adding a log format

1. Add a module under `crates/flightdiff-core/src/format/`.
2. Expose `is_<format>(&[u8]) -> bool` and `parse(&[u8]) -> Result<FlightLog, Error>`.
3. Add one arm to `detect_and_parse`.
4. Add a sample log under `testdata/` with its provenance recorded.

Nothing outside `format/` should need to change.

## Parsing rules

Flight logs are frequently truncated, because the thing writing them crashed.

- A truncated log returns whatever was recovered, with `set_truncated(true)`.
  It is not an error.
- Reserve `Error::Malformed` for damage the reader genuinely cannot step past.
- Never panic on input. Every index into a byte slice needs a bounds check.
  Reports of a panic on a real log are treated as high priority.

## Test data

Only commit logs you have the right to publish. Logs generated from PX4 or
ArduPilot SITL are ideal. Record where each one came from in
`testdata/README.md`. Do not commit anyone's real flight data without written
permission; coordinates in a log are a location history.

## Commit messages

[Conventional Commits](https://www.conventionalcommits.org/): `feat:`, `fix:`,
`perf:`, `docs:`, `refactor:`, `test:`, `chore:`. The changelog is generated
from them.
