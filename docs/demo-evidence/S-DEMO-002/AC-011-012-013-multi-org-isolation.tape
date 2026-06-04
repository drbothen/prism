# S-DEMO-002 / AC-011 + AC-012 + AC-013
# AC-011: 3-org boot (demo-org-a: CS+Armis; demo-org-b: Claroty+Cyberint; demo-org-c: all 4)
#         registers exactly 8 adapters in AdapterRegistry.
# AC-012: Cross-org isolation — demo-org-a querying Claroty returns E-QUERY-032 (code -32602).
# AC-013: DTU multi-tenant — demo-org-a + demo-org-c both reach CrowdStrike DTU clone port.
#
# SID-1 unit substitutes (non-ignored, always run):
#   test_BC_3_2_001_step9a_multi_org_registers_eight_adapters       (AC-011, prism-bin)
#   test_BC_3_2_001_unit_resolve_source_refs_cross_org...e_query_032 (AC-012, prism-query)
#
# BC: BC-3.2.001 (per-org isolation), BC-2.22.001 (boot orchestration).

Output "docs/demo-evidence/S-DEMO-002/AC-011-012-013-multi-org-isolation.gif"
Output "docs/demo-evidence/S-DEMO-002/AC-011-012-013-multi-org-isolation.webm"

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 18
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 40ms

Sleep 2s

Hide
Type "export PATH=/Users/jmagady/.cargo/bin:$PATH"
Enter
Sleep 500ms
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-002"
Enter
Sleep 500ms
Show

# AC-011 SID-1 unit: 8 adapter entries
Type "# AC-011 (unit): 3-org boot registers exactly 8 adapters in AdapterRegistry"
Enter
Sleep 500ms
Type "cargo nextest run -p prism-bin -E 'test(step9a_multi_org_registers_eight)' 2>&1 | grep -E 'PASS|FAIL'"
Enter
Sleep 15s

# AC-012 SID-1 unit: E-QUERY-032
Type "# AC-012 (unit): cross-org sensor query raises E-QUERY-032 (code -32602)"
Enter
Sleep 500ms
Type "cargo nextest run -p prism-query -E 'test(cross_org.*e_query_032)' 2>&1 | grep -E 'PASS|FAIL'"
Enter
Sleep 20s

Type "clear"
Enter
Sleep 500ms

# AC-011 + AC-012 + AC-013 E2E: all three multi-org tests
Type "# AC-011..013 E2E: multi-org boot, E-QUERY-032 isolation, DTU port routing"
Enter
Sleep 500ms
Type "cargo nextest run -p prism-bin --profile e2e --run-ignored all -E 'test(e2e_multi_org) | test(e2e_cross_org) | test(e2e_dtu_multi_tenant)' 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 35s

Sleep 2s
