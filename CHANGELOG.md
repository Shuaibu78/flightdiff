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

[Unreleased]: https://github.com/Shuaibu78/vane/compare/v0.1.0...HEAD
