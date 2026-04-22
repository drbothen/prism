#!/usr/bin/env bash
# test_AC-config_toolchain-files.sh
# Covers config file semantic requirements from Tasks 1-4 and Architecture Compliance Rules:
#  - rust-toolchain.toml pins stable, has rust-src component
#  - rustfmt.toml has all 5 required fields with correct values
#  - clippy.toml has complexity thresholds
#  - kani.toml has exact values from tooling-selection.md
#  - Cargo.toml has [workspace.lints.clippy] with await_holding_lock and unwrap_used
#  - just integration-test, dtu-validate, dtu-start targets exist in Justfile
# FAILS on stubs that are missing semantic values.
set -euo pipefail

WORKTREE="$(cd "$(dirname "$0")/../.." && pwd)"
TAP_COUNT=0
FAIL=0

tap_ok()   { TAP_COUNT=$((TAP_COUNT+1)); echo "ok $TAP_COUNT - $1"; }
tap_fail() { TAP_COUNT=$((TAP_COUNT+1)); echo "not ok $TAP_COUNT - $1"; FAIL=1; }
tap_skip() { TAP_COUNT=$((TAP_COUNT+1)); echo "ok $TAP_COUNT - SKIP $1 # SKIP tool absent"; }

# --- rust-toolchain.toml ---
TOOLCHAIN="$WORKTREE/rust-toolchain.toml"
if [[ -f "$TOOLCHAIN" ]]; then
  tap_ok "rust-toolchain.toml exists"
else
  tap_fail "rust-toolchain.toml missing"
fi

if grep -q 'channel = "stable"' "$TOOLCHAIN"; then
  tap_ok "rust-toolchain.toml pins channel = \"stable\""
else
  tap_fail "rust-toolchain.toml does not pin channel = \"stable\" (architecture compliance rule)"
fi

if grep -q '"rust-src"' "$TOOLCHAIN" || grep -q "'rust-src'" "$TOOLCHAIN" || grep -q 'rust-src' "$TOOLCHAIN"; then
  tap_ok "rust-toolchain.toml includes rust-src component (required for Kani)"
else
  tap_fail "rust-toolchain.toml missing rust-src component"
fi

# rust-toolchain.toml MUST NOT pin nightly as primary channel
if grep -q 'channel = "nightly"' "$TOOLCHAIN"; then
  tap_fail "rust-toolchain.toml pins nightly — must pin stable per architecture compliance"
else
  tap_ok "rust-toolchain.toml does not pin nightly (architecture compliance)"
fi

# --- rustfmt.toml ---
RUSTFMT="$WORKTREE/rustfmt.toml"
if [[ -f "$RUSTFMT" ]]; then
  tap_ok "rustfmt.toml exists"
else
  tap_fail "rustfmt.toml missing"
fi

declare -A RUSTFMT_EXPECTED=(
  ["edition"]='edition = "2021"'
  ["max_width"]='max_width = 100'
  ["use_small_heuristics"]='use_small_heuristics = "Default"'
  ["imports_granularity"]='imports_granularity = "Crate"'
  ["group_imports"]='group_imports = "StdExternalCrate"'
)
for key in "${!RUSTFMT_EXPECTED[@]}"; do
  val="${RUSTFMT_EXPECTED[$key]}"
  if grep -qF "$val" "$RUSTFMT"; then
    tap_ok "rustfmt.toml: $key correct"
  else
    tap_fail "rustfmt.toml: $key incorrect or missing (expected: $val)"
  fi
done

# --- clippy.toml ---
CLIPPY="$WORKTREE/clippy.toml"
if [[ -f "$CLIPPY" ]]; then
  tap_ok "clippy.toml exists"
else
  tap_fail "clippy.toml missing"
fi

if grep -q 'cognitive-complexity-threshold = 30' "$CLIPPY"; then
  tap_ok "clippy.toml: cognitive-complexity-threshold = 30"
else
  tap_fail "clippy.toml: cognitive-complexity-threshold not set to 30"
fi

if grep -q 'too-many-arguments-threshold = 8' "$CLIPPY"; then
  tap_ok "clippy.toml: too-many-arguments-threshold = 8"
else
  tap_fail "clippy.toml: too-many-arguments-threshold not set to 8"
fi

# --- kani.toml ---
KANI="$WORKTREE/kani.toml"
if [[ -f "$KANI" ]]; then
  tap_ok "kani.toml exists"
else
  tap_fail "kani.toml missing"
fi

if grep -q 'default-unwind = 10' "$KANI"; then
  tap_ok "kani.toml: default-unwind = 10"
else
  tap_fail "kani.toml: default-unwind not set to 10 (tooling-selection.md § Kani)"
fi

if grep -q 'timeout = 300' "$KANI"; then
  tap_ok "kani.toml: timeout = 300"
else
  tap_fail "kani.toml: timeout not set to 300"
fi

if grep -q 'memory-limit = 8192' "$KANI"; then
  tap_ok "kani.toml: memory-limit = 8192"
else
  tap_fail "kani.toml: memory-limit not set to 8192"
fi

# --- Cargo.toml workspace lints ---
CARGO="$WORKTREE/Cargo.toml"
if grep -q 'await_holding_lock = "deny"' "$CARGO"; then
  tap_ok "Cargo.toml: await_holding_lock = \"deny\" in [workspace.lints.clippy]"
else
  tap_fail "Cargo.toml: missing await_holding_lock = \"deny\" (Task 3, architecture compliance)"
fi

if grep -q 'unwrap_used = "warn"' "$CARGO"; then
  tap_ok "Cargo.toml: unwrap_used = \"warn\" in [workspace.lints.clippy]"
else
  tap_fail "Cargo.toml: missing unwrap_used = \"warn\""
fi

# --- Justfile DTU + integration targets ---
JUSTFILE="$WORKTREE/Justfile"
for target in "integration-test" "dtu-start" "dtu-validate"; do
  if grep -q "^${target}:" "$JUSTFILE" || grep -q "^${target} " "$JUSTFILE"; then
    tap_ok "Justfile: $target target exists"
  else
    tap_fail "Justfile: $target target missing"
  fi
done

# --- rust-toolchain.toml: rustc --version works (toolchain already pinned in stub) ---
if command -v rustc &>/dev/null; then
  if rustc --version &>/dev/null; then
    tap_ok "rustc --version succeeds (rust-toolchain.toml pin is valid)"
  else
    tap_fail "rustc --version failed — toolchain pin may be broken"
  fi
else
  tap_skip "rustc not on PATH"
fi

echo "1..$TAP_COUNT"
exit $FAIL
