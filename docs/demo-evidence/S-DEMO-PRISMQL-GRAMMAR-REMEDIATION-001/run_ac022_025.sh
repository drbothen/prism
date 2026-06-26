#!/bin/bash
export PATH="/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin"
cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001

echo "=== AC-022/025: Enrich Parse Error Guidance (GRAMMAR-005/015) ==="
echo ""
echo "AC-022: FROM t | enrich threat_score -> guided error (not raw Chumsky dump)"
echo "        'enrich requires a column argument: | enrich <infusion>(<column>)'"
echo "AC-025: Multi-stage pipeline -> same guided error at all pipeline positions"
echo ""
cargo nextest run -p prism-query \
  --test grammar_remediation \
  test_bc_2_11_grammar005_enrich_missing_column_arg_guidance \
  test_bc_2_11_grammar015_enrich_missing_column_arg_multi_stage_guidance \
  test_bc_2_11_obs1_sqlpipe_enrich_missing_column_arg_guided_error \
  2>&1 | grep -E 'PASS|FAIL|Summary|test_bc'
