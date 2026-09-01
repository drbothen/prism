# AC-005 + AC-011 + AC-018 + AC-023: Wire-shape mock tests — class_uid=3004 and Tier-1 presence
#
# AC-005: Mock-path SELECT * on claroty_organization_zones: serialized JSON contains
#         class_uid=3004, name present, raw_extensions present. No standalone Tier-2 root keys.
#         Test: test_BC_2_16_020_claroty_organization_zones_wire_shape_class_uid_3004_mock (prism-bin).
# AC-011: Mock-path SELECT * on claroty_organization_zone_policies: class_uid=3004, name,
#         activity_name present; raw_extensions present containing communication_conditions,
#         related_alerts_ids, applied_zone_pairs keys.
#         Test: test_BC_2_16_020_claroty_organization_zone_policies_wire_shape_class_uid_3004_mock.
# AC-018: Mock-path SELECT * on claroty_organization_firewall_groups: class_uid=3004, name,
#         raw_extensions present containing device_conditions key.
#         Test: test_BC_2_16_021_claroty_organization_firewall_groups_wire_shape_class_uid_3004_mock.
# AC-023: Mock-path SELECT * on claroty_organization_firewall_policies: class_uid=3004, name,
#         activity_name; raw_extensions contains applied_group_pairs (NOT applied_zone_pairs).
#         Test: test_BC_2_16_021_claroty_organization_firewall_policies_wire_shape_class_uid_3004_mock.
#
# Also covers null-not-absent assertions (wire-shape discipline 2026-07-13):
#   test_BC_2_16_020_claroty_organization_zones_wire_shape_serialized_json_null_not_absent
#   test_BC_2_16_020_claroty_organization_zone_policies_wire_shape_serialized_json_null_not_absent
#   test_BC_2_16_021_claroty_organization_firewall_groups_wire_shape_serialized_json_null_not_absent
#   test_BC_2_16_021_claroty_organization_firewall_policies_wire_shape_serialized_json_null_not_absent
#
# Traces to: BC-2.16.020 §Postconditions §1,§3,§6 TV-BC-2.16.020-002,007;
#            BC-2.16.021 §Postconditions §1,§3,§4,§6 TV-BC-2.16.021-002,007

Output "docs/demo-evidence/S-CLAROTY-ORGPOLICY-001/AC-005-011-018-023-wire-shape-mock.gif"
Output "docs/demo-evidence/S-CLAROTY-ORGPOLICY-001/AC-005-011-018-023-wire-shape-mock.webm"

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

Type "# AC-005/011/018/023: wire-shape mock tests — class_uid=3004, Tier-1 present, raw_extensions shape"
Enter
Sleep 400ms

Type `cargo nextest run -p prism-bin -E 'test(BC_2_16_020_claroty_organization_zones_wire_shape) | test(BC_2_16_020_claroty_organization_zone_policies_wire_shape) | test(BC_2_16_021_claroty_organization_firewall_groups_wire_shape) | test(BC_2_16_021_claroty_organization_firewall_policies_wire_shape)' 2>&1 | tail -12`
Enter
Sleep 60s
Sleep 2s
