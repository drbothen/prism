#!/bin/bash
export PATH="/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin"
cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001

echo "=== AC-001/002/003: SQL-to-Pipe Grammar (BC-2.11.020) ==="
echo ""
echo "AC-001: SELECT ... | enrich ... | limit N parses to Ast::SqlPipe with 2 stages"
echo "AC-002: SELECT ... LIMIT 5 | ... | limit 3 -> E-QUERY-040 FORBID-BOTH at plan time"
echo "AC-003: Pure SQL and pure Pipe modes are unchanged (regression guard)"
echo ""
cargo nextest run -p prism-query \
  --test grammar_remediation \
  test_bc_2_11_020_sqlpipe_ast_round_trip \
  test_bc_2_11_020_forbid_both_dual_limit_e_query_040 \
  test_bc_2_11_020_pure_modes_unchanged \
  2>&1 | grep -E 'PASS|FAIL|Summary|test_bc'
