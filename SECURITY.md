# Security Policy

## Supported versions

The most recent minor release receives security fixes.

## Reporting a vulnerability

Do not open a public issue. Use GitHub's private vulnerability reporting on
this repository, or email <devshuaib@gmail.com>.

Expect an acknowledgement within 72 hours.

## Threat model

`flightdiff` parses untrusted binary input. A file that causes a panic, a hang, an
unbounded allocation, or an out-of-bounds read is a security issue here, not a
normal bug. The parsers carry fuzz targets under `fuzz/` for this reason.
