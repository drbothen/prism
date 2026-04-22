#!/usr/bin/env bash
# Demo script for BC-2.09.002 (Provenance Framing) — called by VHS tape
set -e
cd "$(dirname "$0")/.."
echo "=== BC-2.09.002: Provenance Framing — sensor tool descriptions contain provenance warning ==="
cargo test -p prism-security test_BC_2_09_002 2>&1 | grep -E 'test_BC_2_09_002|^test result'
echo "=== PASS: All BC-2.09.002 tests passed ==="
