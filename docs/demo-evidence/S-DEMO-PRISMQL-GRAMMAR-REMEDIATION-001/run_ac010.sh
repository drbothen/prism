#!/bin/bash
export PATH="/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin"
cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001

echo "=== AC-010: normalized_pql on StructuredErrorFields (BC-2.11.023) ==="
echo ""
echo "Demonstrates: D1 mode-bridge error populates normalized_pql rewrite"
echo "in MCP structured error envelope via prism_error_to_structured_call_result"
echo ""
cargo nextest run -p prism-mcp \
  --test mcp_infrastructure \
  test_bc_2_11_023_normalized_pql_on_mode_bridge_error \
  2>&1 | grep -E 'PASS|FAIL|Summary|test_bc'
