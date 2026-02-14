# Quality Scripts

[Korean guide](README_KR.md)

Quality scripts for local quality gate.

Coverage threshold is **85%** (`MIN_LINES=85`).
Coverage output is `target/llvm-cov-target/lcov.info` (used by CI/codecov).

## Files
- `rust-lint-cleanup.sh`: clippy-based lint cleanup script (`--fix`, regular check, dead-code check).
- `rust-coverage.sh`: scenario gate + `cargo llvm-cov` line coverage gate.
- `test-scenarios.csv`: scenario catalog used by the gate.

## Usage
From repo root, run in this order:

```bash
./tools/rust-lint-cleanup.sh
./tools/rust-coverage.sh
```

## Main env vars
- `MIN_LINES` (default: `85`)
- `SCENARIO_CATALOG` (default: `tools/test-scenarios.csv`)
- `MIN_TOTAL` (default: `20`)
- `MIN_AUTOMATED` (default: `17`)
