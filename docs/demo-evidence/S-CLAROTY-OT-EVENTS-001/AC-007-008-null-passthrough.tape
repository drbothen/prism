# AC-007 + AC-008: Null passthrough — absent event_id and detection_time
#
# AC-007: Missing event_id → null finding_info_uid cell; row NOT dropped; time and
#         raw_extensions remain populated; subsequent rows unaffected.
#         Production path: build_column_array within pipeline_result_to_record_batch
#         via SpecDrivenSensorAdapter::fetch (crates/prism-bin/src/spec_driven_adapter.rs)
# AC-008: detection_time null/absent → null time cell; no E-SPEC-018; no pagination halt.
#         Same production path.
#
# Tests run:
#   RG-006: test_BC_2_16_016_claroty_ot_activity_events_required_event_id_absent_produces_null_row
#   RG-007: test_BC_2_16_016_claroty_ot_activity_events_detection_time_null_passthrough
#   RG-011: test_BC_2_16_016_claroty_ot_activity_events_ac007_absent_event_id_null_finding_info_uid_production_path (SAP-3 gate)
#   RG-012: test_BC_2_16_016_claroty_ot_activity_events_ac008_absent_detection_time_null_time_production_path (SAP-3 gate)
# Traces to: BC-2.16.016 §Invariants (REQUIRED push-down; absent-field null passthrough)
#            EC-016-016-001 (event_id absent), EC-016-016-003 (detection_time null)

Output "docs/demo-evidence/S-CLAROTY-OT-EVENTS-001/AC-007-008-null-passthrough.gif"
Output "docs/demo-evidence/S-CLAROTY-OT-EVENTS-001/AC-007-008-null-passthrough.webm"

Require cargo

Set Shell "zsh"
Set WaitTimeout 120s
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 16
Set Width 1200
Set Height 600
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

Type "# AC-007/008: Null passthrough — absent event_id + detection_time (RG-006/007/011/012)"
Enter
Sleep 400ms

Type `cargo nextest run -p prism-bin --test bc_2_16_016_claroty_ot_activity_events_wire_shape -E 'test(ac007) | test(ac008)' 2>&1 | tail -8`
Enter
Sleep 60s
Type `cargo nextest run -p prism-sensors --test bc_2_16_016_claroty_ot_activity_events -E 'test(event_id) | test(detection_time)' 2>&1 | tail -8`
Enter
Sleep 30s
Sleep 2s
