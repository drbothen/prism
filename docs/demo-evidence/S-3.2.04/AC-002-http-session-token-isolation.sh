#!/usr/bin/env bash
# AC-002 demo: HTTP session token registered for org_A is rejected by org_B.
#
# Demonstrates BC-3.2.003 postcondition 2: is_valid_session(org_B, tok_A) == false
# even when tok_A is a valid token for org_A.
set -euo pipefail

WORKTREE=/Users/jmagady/Dev/prism/.worktrees/S-3.2.04

cd "$WORKTREE"

echo "=== AC-002: HTTP session token isolation (BC-3.2.003) ==="
echo ""
echo "--- test: token registered for org_A rejected by org_B ---"
echo ""

cargo test -p prism-dtu-cyberint --features dtu --test multi_tenant \
    test_BC_3_2_003_http_session -- --nocapture 2>/dev/null \
    | grep -E "(running [0-9]+ test|test .* \.\.\. (ok|FAILED)|test result:|PASS|FAIL)" \
    | sed 's/^test /  test /'

echo ""
echo "=== AC-002 PASS: org_B cannot use org_A session token ==="
