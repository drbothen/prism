#!/bin/bash
export PATH="/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin"
cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001

echo "=== AC-004/005: Temporal Grammar — NOW() and INTERVAL (BC-2.11.021) ==="
echo ""
echo "AC-004: NOW() - INTERVAL '24h' parses in SQL, Pipe, and Filter modes"
echo "AC-005: NOW(1), NOW() + INTERVAL, INTERVAL 'bogus' each return E-QUERY-001"
echo ""
cargo nextest run -p prism-query \
  --test grammar_remediation \
  test_bc_2_11_021_now_interval_parses_all_three_modes \
  test_bc_2_11_021_now_error_cases \
  2>&1 | grep -E 'PASS|FAIL|Summary|test_bc'
