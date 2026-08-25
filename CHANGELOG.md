# Changelog

All notable changes are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `vane info` summarises a log.
- `vane diff` reports parameter differences between two logs.
- Format detection for PX4 ULog and ArduPilot DataFlash.
- ULog parameter extraction: the file header, definitions section and
  parameter messages are parsed, so `info` and `diff` report real values.
  Logs cut short mid-message return every parameter recovered before the
  cut and are flagged as truncated. Logs carrying incompatible flag bits
  this reader does not understand are refused rather than half-read.
- ULog logging start time, read from the file header and reported by `info`.
- Synthetic ULog fixtures in `testdata/` with a generator in
  `crates/vane-core/tests/fixtures.rs`. A normal test run compares the
  committed bytes against the generator; `VANE_REGENERATE_FIXTURES=1`
  rewrites them.
- Snapshot tests over real CLI output for `info` and `diff`.
- `docs/demo.svg`, generated from captured program output rather than
  mocked up, replacing a README image that pointed at a file which did
  not exist.

[Unreleased]: https://github.com/Shuaibu78/vane/compare/v0.1.0...HEAD
