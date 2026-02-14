# Quality Scripts

[Korean guide](README_KR.md)

This folder groups quality/coverage helper assets.

## Files
- `rust-lint-cleanup.sh`: clippy-based lint cleanup script (`--fix`, regular check, dead-code check).
- `rust-coverage.sh`: scenario gate + `cargo llvm-cov` line coverage gate.
- `check-scenarios.sh`: validates scenario catalog counts and test refs.
- `test-scenarios.csv`: scenario catalog used by the gate.

## Usage
From repository root:

```bash
./tools/rust-lint-cleanup.sh
./tools/rust-coverage.sh
```

## Main env vars
- `MIN_LINES` (default: `85`)
- `SCENARIO_CATALOG` (default: `tools/test-scenarios.csv`)
- `MIN_TOTAL` (default: `20`)
- `MIN_AUTOMATED` (default: `17`)
