#!/bin/bash
export PATH="/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin"
cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001

echo "=== AC-009/027: Mode-Bridge D1 and D2 Diagnostics (BC-2.11.023) ==="
echo ""
echo "AC-009: D1 — SELECT * FROM t | INVALID triggers mode-bridge message with:"
echo "  (a) stage-keyword enumeration (enrich, where, limit, sort, stats, dedup, fields)"
echo "  (b) numbered alternatives: '1. SQL+pipe composition:' and '2. Pipe mode only:'"
echo "  (c) 'See prismql://reference for the complete grammar.'"
echo "AC-027: D2 — FROM t | ORDER BY produces 'SQL clauses not valid as pipe stages'"
echo ""
cargo nextest run -p prism-query \
  --test grammar_remediation \
  test_bc_2_11_023_mode_bridge_d1_sql_pipe_diagnostic \
  test_bc_2_11_023_mode_bridge_d2_sql_keyword_in_pipe_position \
  2>&1 | grep -E 'PASS|FAIL|Summary|test_bc'
