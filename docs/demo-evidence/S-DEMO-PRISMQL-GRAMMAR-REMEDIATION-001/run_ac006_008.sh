#!/bin/bash
export PATH="/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin"
cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001

echo "=== AC-006/007/008/023/026: prismql://reference content (BC-2.11.022) ==="
echo ""
echo "AC-006: build_reference_content() produces all required sections"
echo "AC-007: CI 3-tier gate (positive / negative / registry parity) passes"
echo "AC-008: build_reference_content(None) returns placeholder without panic"
echo "AC-023: Reference includes IS NOT NULL on JSON-list column semantics"
echo "AC-026: Reference aggregates section documents percentile, distinct_count"
echo ""
cargo nextest run -p prism-mcp \
  --test reference_content \
  2>&1 | grep -E 'PASS|FAIL|Summary|test_bc'
