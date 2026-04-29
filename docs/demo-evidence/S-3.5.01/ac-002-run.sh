#!/usr/bin/env bash
# ac-002-run.sh — Full AC-002 demo: create bad crate, run validator, cleanup
#
# Demonstrates that check-crate-layout.sh detects violations and exits non-zero.
# Output format: "crates/<name>: <rule description>" (BC-3.7.001 postcondition 3).
#
# Traces to: BC-3.7.001 postconditions 2+3, VP-135

set -uo pipefail
set +e
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
WORKTREE_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"

BADROOT=$(mktemp -d)
trap 'rm -rf "${BADROOT}"' EXIT

mkdir -p "${BADROOT}/crates/test-bad-crate"
cat > "${BADROOT}/crates/test-bad-crate/Cargo.toml" <<'EOF'
[package]
name = "test-bad-crate"
version = "0.1.0"
edition = "2021"
EOF
printf 'pub fn hello() {}\n' > "${BADROOT}/crates/test-bad-crate/lib.rs"

echo "=== Synthetic bad crate layout ==="
echo "${BADROOT}/crates/test-bad-crate/"
ls -1 "${BADROOT}/crates/test-bad-crate/"
echo ""
echo "=== Running: check-crate-layout.sh ==="
WORKSPACE_ROOT="${BADROOT}" bash "${WORKTREE_ROOT}/scripts/check-crate-layout.sh"
EXITCODE=$?
echo ""
echo "exit code: ${EXITCODE}"
