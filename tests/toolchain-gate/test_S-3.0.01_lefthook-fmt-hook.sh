#!/usr/bin/env bash
# test_S-3.0.01_lefthook-fmt-hook.sh
# AC-1: lefthook.yml fmt command is "cargo fmt --all --check" (not the broken per-file form).
# AC-2: fmt hook exits non-zero when a staged .rs file has formatting violations.
# AC-3: fmt hook exits 0 when the workspace is clean.
#
# RED GATE: All 3 tests FAIL before lefthook.yml is fixed because the command
# is still "cargo fmt --check {staged_files}" (broken positional-arg form).
set -euo pipefail

WORKTREE="$(cd "$(dirname "$0")/../.." && pwd)"
TAP_COUNT=0
FAIL=0

tap_ok()   { TAP_COUNT=$((TAP_COUNT+1)); echo "ok $TAP_COUNT - $1"; }
tap_fail() { TAP_COUNT=$((TAP_COUNT+1)); printf 'not ok %d - %s\n' "$TAP_COUNT" "$1"; FAIL=1; }
tap_skip() { TAP_COUNT=$((TAP_COUNT+1)); echo "ok $TAP_COUNT - SKIP $1 # SKIP tool absent"; }

LEFTHOOK_YML="$WORKTREE/lefthook.yml"

# ---------------------------------------------------------------------------
# AC-1: Verify the fmt command literal in lefthook.yml
# The correct form is "cargo fmt --all --check" (workspace-wide, no positional files).
# The broken form is "cargo fmt --check {staged_files}" (silently no-ops on stable).
# ---------------------------------------------------------------------------
FMT_RUN=$(grep -A10 '^pre-commit:' "$LEFTHOOK_YML" \
  | grep -A5 'fmt:' \
  | grep 'run:' \
  | sed 's/.*run: *//' \
  | head -1)

EXPECTED_CMD="cargo fmt --all --check"

if [[ "$FMT_RUN" == "$EXPECTED_CMD" ]]; then
  tap_ok "AC-1: lefthook.yml fmt run is 'cargo fmt --all --check'"
else
  tap_fail "AC-1: lefthook.yml fmt run is '$FMT_RUN' — expected '$EXPECTED_CMD' (Red Gate: fix not applied)"
fi

# Also assert that the broken positional-arg form is absent.
if grep -q 'cargo fmt --check {staged_files}' "$LEFTHOOK_YML"; then
  tap_fail "AC-1: lefthook.yml still contains broken 'cargo fmt --check {staged_files}' invocation"
else
  tap_ok "AC-1: broken 'cargo fmt --check {staged_files}' form is absent"
fi

# ---------------------------------------------------------------------------
# AC-2 / AC-3: Functional smoke tests via lefthook run
#
# We create a temporary git fixture repo, copy lefthook.yml into it, then
# run "lefthook run pre-commit" inside it.
# ---------------------------------------------------------------------------

if ! command -v lefthook &>/dev/null; then
  tap_skip "AC-2: lefthook binary not in PATH"
  tap_skip "AC-3: lefthook binary not in PATH"
  echo "1..$TAP_COUNT"
  exit $FAIL
fi

if ! command -v cargo &>/dev/null; then
  tap_skip "AC-2: cargo not in PATH"
  tap_skip "AC-3: cargo not in PATH"
  echo "1..$TAP_COUNT"
  exit $FAIL
fi

# ---- Build fixture repo ----
FIXTURE=$(mktemp -d)
trap 'rm -rf "$FIXTURE"' EXIT

cd "$FIXTURE"
git init -q
git config user.email "test@example.com"
git config user.name "Test"

# Create a minimal Cargo workspace so "cargo fmt --all" has something to check.
cat > Cargo.toml <<'CARGO'
[workspace]
members = ["lib"]
resolver = "2"
CARGO

mkdir lib
cat > lib/Cargo.toml <<'CARGO'
[package]
name = "lib"
version = "0.0.1"
edition = "2021"
CARGO

mkdir lib/src

# Copy the real lefthook.yml from the worktree into the fixture.
cp "$LEFTHOOK_YML" "$FIXTURE/lefthook.yml"

# Install lefthook hooks in the fixture repo (writes .git/hooks/).
lefthook install -f &>/dev/null || true

# ---- AC-3: clean file — hook should exit 0 ----
# Write correctly formatted Rust.
cat > lib/src/lib.rs <<'RUST'
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
RUST

git add lib/src/lib.rs
CLEAN_EXIT=0
lefthook run pre-commit 2>&1 | cat || CLEAN_EXIT=$?

if [[ $CLEAN_EXIT -eq 0 ]]; then
  tap_ok "AC-3: lefthook pre-commit exits 0 on well-formatted workspace"
else
  tap_fail "AC-3: lefthook pre-commit exited $CLEAN_EXIT on well-formatted workspace (expected 0) — Red Gate: fmt command broken"
fi

# ---- AC-2: dirty file — hook should exit non-zero ----
# Overwrite with intentionally misformatted Rust (wrong indentation, extra blank lines).
cat > lib/src/lib.rs <<'RUST'
pub fn add( a:i32,b:i32)->i32{
a+b
}
RUST

git add lib/src/lib.rs
DIRTY_EXIT=0
lefthook run pre-commit 2>&1 | cat || DIRTY_EXIT=$?

if [[ $DIRTY_EXIT -ne 0 ]]; then
  tap_ok "AC-2: lefthook pre-commit exits non-zero on misformatted file (exit $DIRTY_EXIT)"
else
  tap_fail "AC-2: lefthook pre-commit exited 0 on misformatted file — Red Gate: fmt command silently passing (broken {staged_files} form)"
fi

cd "$WORKTREE"

echo "1..$TAP_COUNT"
exit $FAIL
