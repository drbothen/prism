#!/bin/bash
export PATH="/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin"
cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001

echo "=== AC-011/012: Filter-mode + D7 Shared Predicate Grammar (BC-2.11.023) ==="
echo ""
echo "AC-011: Filter mode — bare predicate and source-qualified predicate both produce Ast::Filter"
echo "AC-012: D7 — single build_predicate_parser() used across SQL WHERE, Pipe | where, Filter mode"
echo ""
cargo nextest run -p prism-query \
  --test grammar_remediation \
  test_bc_2_11_023_filter_mode_end_to_end_execution \
  test_bc_2_11_023_d7_shared_predicate_grammar \
  2>&1 | grep -E 'PASS|FAIL|Summary|test_bc'
