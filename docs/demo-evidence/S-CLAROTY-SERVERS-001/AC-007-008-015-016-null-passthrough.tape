# AC-007 + AC-008 + AC-015 + AC-016: Null passthrough and nullable count tests (both tables)
#
# AC-007: Row missing server_name → null row (REQUIRED semantics); no hard error;
#         subsequent rows continue (claroty_servers).
# AC-008: count=null in servers response envelope → empty-page halt; no null-ptr deref;
#         no error raised (claroty_servers pagination edge case).
# AC-015: (1) Row missing server_name → null row (claroty_server_interfaces REQUIRED).
#         (2) Row where interface_name is null but server_name non-null → row NOT dropped;
#         interface_name null in raw_extensions; composite PK degraded, not fatal.
# AC-016: count=null in server_interfaces response envelope → empty-page halt; no error
#         (claroty_server_interfaces pagination edge case).
#
# Tests: RG-007 (test_BC_2_16_018_claroty_servers_required_server_name_absent_produces_null_row)
#        RG-008 (test_BC_2_16_018_claroty_servers_nullable_count_uses_empty_page_halt)
#        RG-015a (test_BC_2_16_019_claroty_server_interfaces_required_server_name_absent_produces_null_row)
#        RG-015b (test_BC_2_16_019_claroty_server_interfaces_null_interface_name_row_not_dropped)
#        RG-016 (test_BC_2_16_019_claroty_server_interfaces_nullable_count_uses_empty_page_halt)
# Traces to: BC-2.16.018 §Invariants, EC-016-018-001/004; BC-2.16.019 §Invariants, EC-016-019-001/002/005

Output "docs/demo-evidence/S-CLAROTY-SERVERS-001/AC-007-008-015-016-null-passthrough.gif"
Output "docs/demo-evidence/S-CLAROTY-SERVERS-001/AC-007-008-015-016-null-passthrough.webm"

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

Type "# AC-007/008/015/016: Null passthrough + nullable count halt (both tables)"
Enter
Sleep 400ms

Type `cargo nextest run -p prism-sensors -E 'test(required_server_name_absent) | test(null_interface_name_row_not_dropped) | test(nullable_count)' 2>&1 | tail -12`
Enter
Sleep 60s
Sleep 2s
