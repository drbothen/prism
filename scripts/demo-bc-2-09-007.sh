#!/usr/bin/env bash
# Demo script for BC-2.09.007 (OutputSchema) — called by VHS tape
set -e
cd "$(dirname "$0")/.."
echo "=== BC-2.09.007: OutputSchema — typed _meta envelope, safety_flags array, no per-field keys ==="
cargo test -p prism-security test_BC_2_09_007 2>&1 | grep -E 'test_BC_2_09_007|^test result'
echo "=== PASS: All BC-2.09.007 tests passed ==="
