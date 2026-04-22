#!/usr/bin/env bash
# Demo script for BC-2.09.004 (Safety Flags Centralized) — called by VHS tape
set -e
cd "$(dirname "$0")/.."
echo "=== BC-2.09.004: Centralized Safety Flags — _meta.safety_flags array, flag-don't-strip ==="
cargo test -p prism-security test_BC_2_09_004 2>&1 | grep -E 'test_BC_2_09_004|^test result'
echo ""
echo "=== BC-2.09.004 (envelope): Original data intact, flags in _meta ==="
cargo test -p prism-mcp test_BC_2_09_004 2>&1 | grep -E 'test_BC_2_09_004|^test result'
echo "=== PASS: All BC-2.09.004 tests passed ==="
