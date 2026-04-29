#!/usr/bin/env bash
# AC-001 demo: All 15 multi_tenant tests GREEN.
#
# Demonstrates BC-3.2.001 + BC-3.2.003: alert_store and session_store
# are both re-keyed by (OrgId, key). The multi_tenant integration suite
# covers all seven acceptance criteria (cross-org isolation, same-token
# independence, reset_for selectivity, OrgId-flip mutation kill).
set -euo pipefail

WORKTREE=/Users/jmagady/Dev/prism/.worktrees/S-3.2.04

cd "$WORKTREE"

echo "=== AC-001: All 15 multi_tenant tests GREEN ==="
echo ""
echo "--- cargo test -p prism-dtu-cyberint --features dtu --test multi_tenant ---"
echo ""

cargo test -p prism-dtu-cyberint --features dtu --test multi_tenant 2>&1 \
    | grep -E "(running [0-9]+ test|test .* \.\.\. (ok|FAILED)|test result:)" \
    | sed 's/^test /  test /'

echo ""
echo "=== AC-001 PASS: all 15 multi_tenant tests green ==="
