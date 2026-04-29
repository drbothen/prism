#!/usr/bin/env bash
# ac-002-setup.sh — Creates and tears down the synthetic bad-crate for AC-002 demo
#
# Usage:
#   source docs/demo-evidence/S-3.5.01/ac-002-setup.sh   # sets BADROOT
#   unset BADROOT                                         # after demo
#
# This script is part of the AC-002 demo evidence (not production code).

set -uo pipefail

BADROOT=$(mktemp -d)
export BADROOT
mkdir -p "${BADROOT}/crates/test-bad-crate"
cat > "${BADROOT}/crates/test-bad-crate/Cargo.toml" <<'EOF'
[package]
name = "test-bad-crate"
version = "0.1.0"
edition = "2021"
EOF
cat > "${BADROOT}/crates/test-bad-crate/lib.rs" <<'EOF'
pub fn hello() {}
EOF
echo "Synthetic bad crate created at: ${BADROOT}/crates/test-bad-crate/"
echo "  - lib.rs is at crate root (violates Rule 1 + Rule 2)"
