#!/bin/bash
export PATH="/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin"
cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001

echo "=== AC-015/016: MCP Prompts Fast-Return (BC-2.10.016) ==="
echo ""
echo "AC-015: All prompts/get calls return within 5 seconds (BLOCKER-003 hang fixed)"
echo "AC-016: Missing required argument investigate_host.hostname -> structured error within 5s"
echo ""
cargo nextest run -p prism-mcp \
  --test mcp_infrastructure \
  test_bc_2_10_016_prompts_fast_return_within_5s \
  test_bc_2_10_016_missing_required_arg_fast_error \
  test_bc_2_10_016_get_prompt_full_transport_dispatch \
  test_high1_investigate_host_full_transport_dispatch \
  test_high1_missing_required_arg_via_full_transport_no_hang \
  2>&1 | grep -E 'PASS|FAIL|Summary|test_bc|test_high'
