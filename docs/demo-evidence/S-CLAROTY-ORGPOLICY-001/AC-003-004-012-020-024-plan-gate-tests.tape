# AC-003 + AC-004 + AC-012 + AC-020 + AC-024: E-QUERY-038 plan-gate tests (Tier-2 and raw Tier-1 names)
#
# AC-003: SELECT zone_source FROM claroty_organization_zones raises E-QUERY-038;
#         available_columns contains raw_extensions, name, comment, status_code, actor_user_name
#         but NOT zone_source. Authoritative gate: prism-bin E2E via QueryEngine::execute (RG-003).
# AC-004: SELECT zone_name FROM claroty_organization_zones raises E-QUERY-038;
#         available_columns has 'name' (Arrow form) but NOT 'zone_name' (raw TOML form).
#         Defense-in-depth gate (prism-sensors unit): RG-004.
# AC-012: SELECT applied_zone_pairs FROM claroty_organization_zone_policies raises E-QUERY-038;
#         available_columns has raw_extensions but NOT applied_zone_pairs.
#         Authoritative gate: prism-bin E2E (RG-012).
# AC-020: SELECT firewall_group_source FROM claroty_organization_firewall_groups raises E-QUERY-038;
#         available_columns has raw_extensions but NOT firewall_group_source.
#         Authoritative gate: prism-bin E2E (RG-020).
# AC-024: SELECT applied_group_pairs FROM claroty_organization_firewall_policies raises E-QUERY-038;
#         available_columns has raw_extensions but NOT applied_group_pairs.
#         Authoritative gate: prism-bin E2E (RG-024).
#
# Tests (prism-bin authoritative, via QueryEngine::execute):
#   RG-003 (test_BC_2_16_020_claroty_organization_zones_e2e_e_query_038_tier2_column)
#   RG-012 (test_BC_2_16_020_claroty_organization_zone_policies_e2e_e_query_038_tier2_column)
#   RG-020 (test_BC_2_16_021_claroty_organization_firewall_groups_e2e_e_query_038_tier2_column)
#   RG-024 (test_BC_2_16_021_claroty_organization_firewall_policies_e2e_e_query_038_tier2_column)
# Defense-in-depth (prism-sensors unit):
#   RG-004 (test_BC_2_16_020_claroty_organization_zones_tier1_raw_toml_name_raises_e_query_038)
#   test_BC_2_16_020_claroty_organization_zones_tier2_column_raises_e_query_038
#   test_BC_2_16_020_claroty_organization_zone_policies_applied_zone_pairs_raises_e_query_038
#   test_BC_2_16_021_claroty_organization_firewall_groups_tier2_column_raises_e_query_038
#   test_BC_2_16_021_claroty_organization_firewall_policies_applied_group_pairs_raises_e_query_038
# Traces to: BC-2.16.020 §Invariants EC-016-020-005,006; BC-2.16.021 §Invariants EC-016-021-005,007

Output "docs/demo-evidence/S-CLAROTY-ORGPOLICY-001/AC-003-004-012-020-024-plan-gate-tests.gif"
Output "docs/demo-evidence/S-CLAROTY-ORGPOLICY-001/AC-003-004-012-020-024-plan-gate-tests.webm"

Require cargo

Set Shell "zsh"
Set WaitTimeout 120s
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 16
Set Width 1200
Set Height 500
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 40ms

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-CLAROTY-ORGPOLICY-001"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 0.5s
Show

Type "# AC-003/004/012/020/024: E-QUERY-038 plan gates — Tier-2 and raw Tier-1 name rejection"
Enter
Sleep 400ms

Type `cargo nextest run -p prism-bin -E 'test(BC_2_16_020_claroty_organization_zones_e2e) | test(BC_2_16_020_claroty_organization_zone_policies_e2e) | test(BC_2_16_021_claroty_organization_firewall_groups_e2e) | test(BC_2_16_021_claroty_organization_firewall_policies_e2e)' 2>&1 | tail -10`
Enter
Sleep 60s
Sleep 2s
