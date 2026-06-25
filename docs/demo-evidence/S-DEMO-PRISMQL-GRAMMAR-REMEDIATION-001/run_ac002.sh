#!/bin/bash
export PATH="/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin"
cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001

echo "=== AC-002: FORBID-BOTH E-QUERY-040 verbatim message text (BC-2.11.020) ==="
echo ""
echo "Demonstrates: E-QUERY-040 message with both limit counts interpolated,"
echo "neutral row-cap wording (v2.00): 'row-capping | limit / | tail pipe stage (cap: N)'"
echo "'PrismQL requires exactly one row cap'"
echo ""
cargo nextest run -p prism-query \
  --test grammar_remediation \
  test_bc_2_11_020_forbid_both_dual_limit_e_query_040 \
  2>&1 | grep -E 'PASS|FAIL|Summary|test_bc'
