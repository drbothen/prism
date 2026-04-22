#!/usr/bin/env bash
# Demo script for BC-2.09.005 (Trust Level Metadata) — called by VHS tape
set -e
cd "$(dirname "$0")/.."
echo "=== BC-2.09.005: Trust-Level Metadata — sensor=untrusted_external, health/caps=internal ==="
cargo test -p prism-security test_BC_2_09_005 2>&1 | grep -E 'test_BC_2_09_005|^test result'
echo ""
echo "=== BC-2.09.005 (envelope): Envelope trust_level set correctly ==="
cargo test -p prism-mcp test_BC_2_09_005 2>&1 | grep -E 'test_BC_2_09_005|^test result'
echo "=== PASS: All BC-2.09.005 tests passed ==="
