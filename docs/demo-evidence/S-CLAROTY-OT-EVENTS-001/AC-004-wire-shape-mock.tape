# AC-004: Wire-shape: class_uid=2004, Tier-1 field presence, raw_extensions in JSON output
#
# AC-004: SELECT * serialized JSON contains class_uid=2004, finding_info_uid,
#         raw_extensions with network 5-tuple (no standalone Tier-2 keys).
# Also covers:
#   AC-006 (EC-002-WIRE): related_alert_ids serialized as native JSON array not string.
#
# Tests run:
#   test_BC_2_16_016_claroty_ot_activity_events_wire_shape_class_uid_2004_mock (wire companion)
#   test_BC_2_16_016_claroty_ot_activity_events_ec002_related_alert_ids_native_json_array (RG-010)
#   (Live wire shape: AC-004-005-006-live-queries.txt — quiescent 0 events; mock covers shape)
# Traces to: BC-2.16.016 §Postconditions §1 (class_uid), §2 (Tier-1/Tier-2 wire representation)

Output "docs/demo-evidence/S-CLAROTY-OT-EVENTS-001/AC-004-wire-shape-mock.gif"
Output "docs/demo-evidence/S-CLAROTY-OT-EVENTS-001/AC-004-wire-shape-mock.webm"

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

Type "# AC-004: class_uid=2004 wire shape + EC-002 JSON array (wire companion + RG-010)"
Enter
Sleep 400ms

Type `cargo nextest run -p prism-bin --test bc_2_16_016_claroty_ot_activity_events_wire_shape -E 'test(class_uid) | test(ec002)' 2>&1 | tail -8`
Enter
Sleep 60s
Sleep 2s
