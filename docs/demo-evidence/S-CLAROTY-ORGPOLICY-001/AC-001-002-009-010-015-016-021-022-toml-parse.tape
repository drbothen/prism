# AC-001 + AC-002 + AC-009 + AC-010 + AC-015 + AC-016 + AC-021 + AC-022:
# TOML blocks parse without validation error; column counts correct;
# Tier-1 columns classified correctly (all 4 org-policy tables).
#
# AC-001: SpecLoader::parse on claroty.sensor.toml returns Ok(SensorSpec);
#         claroty_organization_zones table present; 11 ColumnSpec entries;
#         pagination offset_limit 1000; response_path $.organization_zones.
# AC-002: Exactly 4 Tier-1 columns for zones (zone_name→name REQUIRED,
#         zone_description→comment, enabled→status_code, updated_by→actor.user.name);
#         7 columns have ocsf_field == None (Tier-2 → raw_extensions).
# AC-009: SpecLoader::parse returns Ok; claroty_organization_zone_policies present;
#         13 ColumnSpec entries; field name 'last_updated' (with 'd').
# AC-010: Exactly 4 Tier-1 columns for zone_policies (policy_name→name REQUIRED,
#         policy_action→activity_name, policy_notes→comment, updated_by→actor.user.name);
#         9 Tier-2 columns (incl. 3 Json).
# AC-015: SpecLoader::parse returns Ok; claroty_organization_firewall_groups present;
#         11 ColumnSpec entries; path_template /api/v1/organization_fw_groups/ (abbreviated);
#         response_path $.organization_firewall_groups (full spelling).
# AC-016: Exactly 4 Tier-1 columns for fw_groups (firewall_group_name→name REQUIRED,
#         firewall_group_description→comment, enabled→status_code, updated_by→actor.user.name);
#         7 Tier-2.
# AC-021: SpecLoader::parse returns Ok; claroty_organization_firewall_policies present;
#         13 ColumnSpec entries; path_template /api/v1/organization_fw_group_policies/.
# AC-022: Exactly 4 Tier-1 columns for fw_policies (policy_name→name REQUIRED,
#         policy_action→activity_name, policy_notes→comment, updated_by→actor.user.name);
#         9 Tier-2 (incl. applied_group_pairs NOT applied_zone_pairs).
#
# Tests: RG-001 (test_BC_2_16_020_claroty_organization_zones_toml_block_parses)
#        RG-002 (test_BC_2_16_020_claroty_organization_zones_tier1_columns_four_with_ocsf_field)
#        RG-009 (test_BC_2_16_020_claroty_organization_zone_policies_toml_block_parses)
#        RG-010 (test_BC_2_16_020_claroty_organization_zone_policies_tier1_columns_four_with_ocsf_field)
#        RG-015 (test_BC_2_16_021_claroty_organization_firewall_groups_toml_block_parses)
#        RG-016 (test_BC_2_16_021_claroty_organization_firewall_groups_tier1_columns_four_with_ocsf_field)
#        RG-021 (test_BC_2_16_021_claroty_organization_firewall_policies_toml_block_parses)
#        RG-022 (test_BC_2_16_021_claroty_organization_firewall_policies_tier1_columns_four_with_ocsf_field)
# Traces to: BC-2.16.020 §Postconditions §1,§2,§3,§4; BC-2.16.021 §Postconditions §1,§2,§3,§4

Output "docs/demo-evidence/S-CLAROTY-ORGPOLICY-001/AC-001-002-009-010-015-016-021-022-toml-parse.gif"
Output "docs/demo-evidence/S-CLAROTY-ORGPOLICY-001/AC-001-002-009-010-015-016-021-022-toml-parse.webm"

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

Type "# AC-001/002/009/010/015/016/021/022: TOML parse + Tier-1 classification — all 4 org-policy tables"
Enter
Sleep 400ms

Type `cargo nextest run -p prism-sensors -E 'test(BC_2_16_020_claroty_organization_zones_toml_block) | test(BC_2_16_020_claroty_organization_zones_tier1) | test(BC_2_16_020_claroty_organization_zone_policies_toml_block) | test(BC_2_16_020_claroty_organization_zone_policies_tier1) | test(BC_2_16_021_claroty_organization_firewall_groups_toml_block) | test(BC_2_16_021_claroty_organization_firewall_groups_tier1) | test(BC_2_16_021_claroty_organization_firewall_policies_toml_block) | test(BC_2_16_021_claroty_organization_firewall_policies_tier1)' 2>&1 | tail -14`
Enter
Sleep 60s
Sleep 2s
