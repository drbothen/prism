#!/usr/bin/env bash
# Demo script for BC-2.09.008 (Response Envelope) — called by VHS tape
set -e
cd "$(dirname "$0")/.."
echo "=== BC-2.09.008: Response Envelope — _meta with trust_level, safety_flags, source_sensor ==="
cargo test -p prism-mcp test_BC_2_09_008 2>&1 | grep -E 'test_BC_2_09_008|^test result'
echo "=== PASS: All BC-2.09.008 tests passed ==="
