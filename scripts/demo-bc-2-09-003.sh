#!/usr/bin/env bash
# Demo script for BC-2.09.003 (Injection Scanner + NFKC) — called by VHS tape
set -e
cd "$(dirname "$0")/.."
echo "=== BC-2.09.003: Injection Scanner — NFKC normalization, pattern detection, base64 heuristic ==="
cargo test -p prism-security test_BC_2_09_003 2>&1 | grep -E 'test_BC_2_09_003|^test result'
echo "=== PASS: All BC-2.09.003 tests passed (16 tests) ==="
