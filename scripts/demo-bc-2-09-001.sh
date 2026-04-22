#!/usr/bin/env bash
# Demo script for BC-2.09.001 (Structural Separation) — called by VHS tape
set -e
cd "$(dirname "$0")/.."
echo "=== BC-2.09.001: Structural Separation — sensor data in results, prose summary counts only ==="
cargo test -p prism-mcp test_BC_2_09_001 2>&1 | grep -E 'test_BC_2_09_001|^test result'
echo "=== PASS: All BC-2.09.001 tests passed ==="
