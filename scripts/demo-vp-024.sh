#!/usr/bin/env bash
# Demo script for VP-024 (proptest: InjectionScanner detects known patterns) — called by VHS tape
set -e
cd "$(dirname "$0")/.."
echo "=== VP-024: Proptest — InjectionScanner detects known injection patterns in noisy strings ==="
cargo test -p prism-security test_VP_024 2>&1 | grep -E 'test_VP_024|^test result'
echo "=== PASS: VP-024 proptest passed (4 property-based tests) ==="
