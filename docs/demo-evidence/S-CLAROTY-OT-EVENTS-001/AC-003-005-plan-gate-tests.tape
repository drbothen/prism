# AC-003 + AC-005: Tier-2 plan-gate (E-QUERY-038) test suite evidence
#
# AC-003: SELECT source_ip raises E-QUERY-038; available_columns has raw_extensions,
#         not source_ip (RG-003 authoritative end-to-end gate via prism-bin).
# AC-005: SELECT detection_time raises E-QUERY-038 (raw TOML name, not Arrow name);
#         use `time` instead (RG-009 via prism-bin QueryEngine::execute).
#
# Tests run:
#   RG-003: test_BC_2_16_016_claroty_ot_activity_events_tier2_source_ip_raises_e_query_038
#   RG-009: test_BC_2_16_016_claroty_ot_activity_events_raw_toml_name_detection_time_raises_e_query_038
# Traces to: BC-2.16.016 §Invariants (Tier-2 plan-gate), EC-009 (detection_time raw name)

Output "docs/demo-evidence/S-CLAROTY-OT-EVENTS-001/AC-003-005-plan-gate-tests.gif"
Output "docs/demo-evidence/S-CLAROTY-OT-EVENTS-001/AC-003-005-plan-gate-tests.webm"

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
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-CLAROTY-OT-EVENTS-001"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 0.5s
Show

Type "# AC-003/005: E-QUERY-038 plan-gate tests (RG-003, RG-009 via prism-bin)"
Enter
Sleep 400ms

Type `cargo nextest run -p prism-bin --test bc_2_16_016_claroty_ot_activity_events_wire_shape -E 'test(tier2_source_ip) | test(detection_time)' 2>&1 | tail -8`
Enter
Sleep 60s
Sleep 2s
