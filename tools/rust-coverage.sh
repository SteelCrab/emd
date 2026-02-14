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
MIN_TOTAL="${MIN_TOTAL:-20}"
MIN_AUTOMATED="${MIN_AUTOMATED:-17}"
COVERAGE_LCOV="${PROJECT_ROOT}/target/llvm-cov-target/lcov.info"

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo not found."
  exit 1
fi

if ! command -v cargo-llvm-cov >/dev/null 2>&1; then
  echo "cargo-llvm-cov not found. Install with:"
  echo "  cargo install cargo-llvm-cov --locked"
  exit 1
fi

if [[ ! -f "$SCENARIO_CATALOG" ]]; then
  echo "Scenario catalog not found: $SCENARIO_CATALOG"
  exit 1
fi

find_test_ref() {
  local test_ref="$1"
  if [[ ! "$test_ref" =~ ^[A-Za-z_][A-Za-z0-9_]*$ ]]; then
    return 1
  fi

  local fn_literal="fn ${test_ref}("
  if command -v rg >/dev/null 2>&1; then
    rg -n --fixed-strings "$fn_literal" "${PROJECT_ROOT}/src" >/dev/null
  else
    grep -RsnF "$fn_literal" "${PROJECT_ROOT}/src" >/dev/null
  fi
}

echo "=== scenario gate ==="
echo "Scenario catalog: $SCENARIO_CATALOG"
total="$(awk -F, 'NR > 1 && $1 != "" { count++ } END { print count + 0 }' "$SCENARIO_CATALOG")"
automated="$(awk -F, 'NR > 1 && tolower($5) == "yes" { count++ } END { print count + 0 }' "$SCENARIO_CATALOG")"
echo "Total scenarios: $total (required: >= $MIN_TOTAL)"
echo "Automated scenarios: $automated (required: >= $MIN_AUTOMATED)"

if (( total < MIN_TOTAL )); then
  echo "Scenario total gate failed."
  exit 1
fi

if (( automated < MIN_AUTOMATED )); then
  echo "Scenario automated gate failed."
  exit 1
fi

missing_refs=()
while IFS=, read -r id service priority test_type automated_flag test_ref description; do
  [[ "$id" == "id" ]] && continue

  automated_flag_lower="$(printf '%s' "$automated_flag" | tr '[:upper:]' '[:lower:]')"
  if [[ "$automated_flag_lower" != "yes" ]]; then
    continue
  fi

  if [[ -z "$test_ref" ]]; then
    missing_refs+=("${id}:missing_test_ref")
    continue
  fi

  if ! find_test_ref "$test_ref"; then
    missing_refs+=("${id}:${test_ref}")
  fi
done < "$SCENARIO_CATALOG"

if (( ${#missing_refs[@]} > 0 )); then
  echo "Missing automated scenario test refs:"
  for missing in "${missing_refs[@]}"; do
    echo "  - $missing"
  done
  exit 1
fi

echo "Scenario gate passed."

echo "=== global coverage gate ==="
mkdir -p "$(dirname "${COVERAGE_LCOV}")"
cargo llvm-cov \
  --manifest-path "${PROJECT_ROOT}/Cargo.toml" \
  --workspace \
  --all-features \
  --all-targets \
  --fail-under-lines "$MIN_LINES" \
  --lcov --output-path "${COVERAGE_LCOV}"

echo "Coverage report: ${COVERAGE_LCOV}"

echo "Global coverage gate passed."
