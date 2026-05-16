# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-05-16

### Added

- Async `MarketEventStream` over NDJSON files (snapshots, depth batches, trades).
- `MarketEvent` and related types with Serde deserialization.
- `MarketFlowError` for I/O and JSON parse failures.
- Example `read_feed`, Criterion benchmarks, and `just` recipes.

### Notes

- Early release: use at your own risk; patch releases may change parsing details.
- Wire format is documented in the README; unknown `type` values fail deserialization.

[0.1.0]: https://github.com/avyactjain/market-flow/releases/tag/v0.1.0
