#!/bin/bash
export PATH="/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin"
cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001

echo "=== AC-013/014: list_capabilities consults OrgRegistry (BC-2.10.015) ==="
echo ""
echo "AC-013: FeatureFlagEvaluator consults OrgRegistry::slug_exists()"
echo "        org-c registered -> client_registered: true"
echo "AC-014: Non-existent org -> client_registered: false"
echo ""
cargo nextest run -p prism-mcp \
  --test mcp_infrastructure \
  test_bc_2_10_015_client_registered_true_from_org_registry \
  test_bc_2_10_015_demo_provisioned_org_registered \
  2>&1 | grep -E 'PASS|FAIL|Summary|test_bc'
