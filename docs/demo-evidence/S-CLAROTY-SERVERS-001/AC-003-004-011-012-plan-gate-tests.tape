# AC-003 + AC-004 + AC-011 + AC-012: E-QUERY-038 plan-gate tests (both tables)
#
# AC-003: SELECT server_location FROM claroty_servers raises E-QUERY-038 at plan time;
#         available_columns contains raw_extensions, device_name, status_code, class_uid, _sensor
#         and does NOT contain server_location.
# AC-004: SELECT server_status FROM claroty_servers raises E-QUERY-038; available has status_code
#         not server_status. SELECT server_name raises E-QUERY-038; available has device_name
#         not server_name. (Raw Tier-1 TOML names are rejected; Arrow names are enforced.)
# AC-011: SELECT interface_name FROM claroty_server_interfaces raises E-QUERY-038; composite PK
#         element interface_name is Tier-2 in raw_extensions, not a standalone Arrow column.
# AC-012: SELECT interface_status FROM claroty_server_interfaces raises E-QUERY-038;
#         available has status_code not interface_status. (Raw Tier-1 TOML name rejected.)
#
# Tests: RG-003 (test_BC_2_16_018_claroty_servers_e2e_e_query_038_tier2_column — prism-bin E2E)
#        RG-004 (test_BC_2_16_018_claroty_servers_tier1_raw_toml_name_raises_e_query_038)
#        RG-011 (test_BC_2_16_019_claroty_server_interfaces_e2e_e_query_038_tier2_column — prism-bin E2E)
#        RG-012 (test_BC_2_16_019_claroty_server_interfaces_interface_status_raw_name_raises_e_query_038)
# Traces to: BC-2.16.018 §Invariants, EC-016-018-005; BC-2.16.019 §Invariants, EC-016-019-004, EC-016-019-006

Output "docs/demo-evidence/S-CLAROTY-SERVERS-001/AC-003-004-011-012-plan-gate-tests.gif"
Output "docs/demo-evidence/S-CLAROTY-SERVERS-001/AC-003-004-011-012-plan-gate-tests.webm"

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
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-CLAROTY-SERVERS-001"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 0.5s
Show

Type "# AC-003/004/011/012: E-QUERY-038 plan-gate (Tier-2 + raw Tier-1 names rejected)"
Enter
Sleep 400ms

Type `cargo nextest run -p prism-bin --test bc_2_16_018_claroty_servers_wire_shape --test bc_2_16_019_claroty_server_interfaces_wire_shape -E 'test(e_query_038)' 2>&1 | tail -10`
Enter
Sleep 60s
Sleep 2s
