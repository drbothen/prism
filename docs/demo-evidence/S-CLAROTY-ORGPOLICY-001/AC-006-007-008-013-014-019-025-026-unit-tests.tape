# AC-006 + AC-007 + AC-008 + AC-013 + AC-014 + AC-019 + AC-025 + AC-026:
# JSON column serialization, REQUIRED field null-row, nullable count pagination
# (all unit tests in prism-sensors; no live dependency)
#
# AC-006: device_conditions column_type=json in zones TOML; raw_extensions["device_conditions"]
#         is a JSON array (not a string token); empty array → [] not null.
#         Test: test_BC_2_16_020_claroty_organization_zones_device_conditions_json_not_string
# AC-007: zone_name REQUIRED; absent/null zone_name → null row, no hard error, rows continue.
#         Test: test_BC_2_16_020_claroty_organization_zones_required_zone_name_absent_produces_null_row
# AC-008: count=null in organization_zones envelope → empty-page halt; no null-ptr error.
#         Test: test_BC_2_16_020_claroty_organization_zones_nullable_count_uses_empty_page_halt
# AC-013: policy_name REQUIRED in zone_policies; absent → null row, no hard error.
#         Test: test_BC_2_16_020_claroty_organization_zone_policies_required_policy_name_absent_produces_null_row
# AC-014: communication_conditions, related_alerts_ids, applied_zone_pairs in zone_policies
#         are column_type=json; each serializes as JSON array (not string) in raw_extensions.
#         Test: test_BC_2_16_020_claroty_organization_zone_policies_json_columns_not_stringified
# AC-019: firewall_group_name REQUIRED; absent → null row, no hard error.
#         Test: test_BC_2_16_021_claroty_organization_firewall_groups_required_fwgroupname_absent_produces_null_row
# AC-025: policy_name REQUIRED in fw_policies → null row; count=null → empty-page halt.
#         Tests: test_BC_2_16_021_claroty_organization_firewall_policies_required_policy_name_absent_produces_null_row
#                test_BC_2_16_021_claroty_organization_firewall_policies_nullable_count_uses_empty_page_halt
# AC-026: communication_conditions, related_alerts_ids, applied_group_pairs in fw_policies
#         are column_type=json; applied_group_pairs NOT applied_zone_pairs.
#         Test: test_BC_2_16_021_claroty_organization_firewall_policies_json_columns_not_stringified
#
# Traces to: BC-2.16.020 §Postconditions §6; §Invariants EC-016-020-001..004;
#            BC-2.16.021 §Postconditions §6; §Invariants EC-016-021-001..004,010

Output "docs/demo-evidence/S-CLAROTY-ORGPOLICY-001/AC-006-007-008-013-014-019-025-026-unit-tests.gif"
Output "docs/demo-evidence/S-CLAROTY-ORGPOLICY-001/AC-006-007-008-013-014-019-025-026-unit-tests.webm"

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

Type "# AC-006/007/008/013/014/019/025/026: Json columns, REQUIRED fields, nullable count"
Enter
Sleep 400ms

Type `cargo nextest run -p prism-sensors -E 'test(BC_2_16_020_claroty_organization_zones_device_conditions) | test(BC_2_16_020_claroty_organization_zones_required) | test(BC_2_16_020_claroty_organization_zones_nullable) | test(BC_2_16_020_claroty_organization_zone_policies_required) | test(BC_2_16_020_claroty_organization_zone_policies_json) | test(BC_2_16_021_claroty_organization_firewall_groups_required) | test(BC_2_16_021_claroty_organization_firewall_policies_required) | test(BC_2_16_021_claroty_organization_firewall_policies_nullable) | test(BC_2_16_021_claroty_organization_firewall_policies_json)' 2>&1 | tail -14`
Enter
Sleep 60s
Sleep 2s
