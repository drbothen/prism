#!/usr/bin/env bash
# Demo script for BC-2.09.006 (Tool Description Security Warnings) — called by VHS tape
set -e
cd "$(dirname "$0")/.."
echo "=== BC-2.09.006: Tool Description Warnings — 9 required sections, DATA TRUST LEVEL, SECURITY NOTE ==="
cargo test -p prism-security test_BC_2_09_006 2>&1 | grep -E 'test_BC_2_09_006|^test result'
echo ""
echo "=== BC-2.09.006 (registrar): Framework appends missing sections idempotently ==="
cargo test -p prism-mcp test_BC_2_09_006 2>&1 | grep -E 'test_BC_2_09_006|^test result'
echo "=== PASS: All BC-2.09.006 tests passed ==="
