#!/usr/bin/env bash
# Description:
#   Coverage gate for large-scale tests.
#   - Scenario gate: 20 total / 17 automated
#   - Global workspace line coverage gate: cargo llvm-cov >= 85%
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
find_project_root() {
  local dir="$SCRIPT_DIR"
  while [[ "$dir" != "/" ]]; do
    if [[ -f "$dir/Cargo.toml" ]]; then
      printf '%s\n' "$dir"
      return 0
    fi
    dir="$(dirname "$dir")"
  done
  return 1
}

PROJECT_ROOT="$(find_project_root)" || {
  echo "Cargo.toml not found from script path: $SCRIPT_DIR"
  exit 1
}
SCENARIO_CATALOG="${SCENARIO_CATALOG:-${SCRIPT_DIR}/test-scenarios.csv}"
MIN_LINES="${MIN_LINES:-85}"

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo not found."
  exit 1
fi

if ! command -v cargo-llvm-cov >/dev/null 2>&1; then
  echo "cargo-llvm-cov not found. Install with:"
  echo "  cargo install cargo-llvm-cov --locked"
  exit 1
fi

echo "=== scenario gate ==="
bash "${SCRIPT_DIR}/check-scenarios.sh" "$SCENARIO_CATALOG"

echo "=== global coverage gate ==="
cargo llvm-cov \
  --manifest-path "${PROJECT_ROOT}/Cargo.toml" \
  --workspace \
  --all-features \
  --all-targets \
  --fail-under-lines "$MIN_LINES"

echo "Global coverage gate passed."
