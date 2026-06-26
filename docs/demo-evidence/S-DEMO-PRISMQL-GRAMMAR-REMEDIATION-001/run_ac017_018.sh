#!/bin/bash
export PATH="/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin"
cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001

echo "=== AC-017/018: NOT_YET_AVAILABLE Fast-Fail (BC-2.10.017) ==="
echo ""
echo "AC-017: list_infusions / plugin_status / infusion_status -> -32003 within 1s"
echo "   Test injects slow AuditWriter; guard short-circuits BEFORE audit await"
echo "AC-018: no NOT_YET_AVAILABLE path awaits blocking audit I/O"
echo ""
cargo nextest run -p prism-mcp \
  --test mcp_infrastructure \
  test_bc_2_10_017_not_yet_available_fast_fail_under_1s \
  test_bc_2_10_017_not_yet_available_guard_precedes_audit \
  test_bc_2_10_017_sibling_handlers_guard_precedes_audit \
  2>&1 | grep -E 'PASS|FAIL|Summary|test_bc'
