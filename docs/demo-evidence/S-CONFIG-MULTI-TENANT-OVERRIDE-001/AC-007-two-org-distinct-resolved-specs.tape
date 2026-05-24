Output AC-007-two-org-distinct-resolved-specs.gif
Output AC-007-two-org-distinct-resolved-specs.webm

Require cargo

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set Padding 20
Set FontFamily "FiraCode Nerd Font Mono"

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-CONFIG-MULTI-TENANT-OVERRIDE-001"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-007: Two-org overlays produce distinct ResolvedSensorSpec entries (BC-2.06.012 EC-012-006)"
Enter
Sleep 500ms

Type "# Fixture: customers/acme/armis + customers/contoso/armis -> 2 independent entries"
Enter
Sleep 300ms
Type "# Asserts: distinct base_urls per org | identical [[tables]] schema (INV-OVL-001)"
Enter
Sleep 500ms

Type "cargo nextest run -p prism-spec-engine -E 'test(test_S_CONFIG_MULTI_TENANT_OVERRIDE_001_007_two_org_overlays_produce_distinct_resolved_specs)' 2>&1 | tail -8"
Enter
Sleep 10s
