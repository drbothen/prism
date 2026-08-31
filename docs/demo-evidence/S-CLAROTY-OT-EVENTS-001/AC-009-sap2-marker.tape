# AC-009: SAP-2 N/A documented — no DTU exists; live-only validation
#
# The test file contains a visible constant: SAP2_STATUS = "N/A: no DTU; deferred D-2200"
# This marker ensures adversarial review can verify the SAP-2 N/A claim without
# spawning false DTU-parity findings.
#
# Tests run:
#   RG-008: test_BC_2_16_016_claroty_ot_activity_events_sap2_na_documented
# Traces to: BC-2.16.016 §Postconditions §4 (SAP-2 DTU parity N/A)

Output "docs/demo-evidence/S-CLAROTY-OT-EVENTS-001/AC-009-sap2-marker.gif"
Output "docs/demo-evidence/S-CLAROTY-OT-EVENTS-001/AC-009-sap2-marker.webm"

Require cargo

Set Shell "zsh"
Set WaitTimeout 120s
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 16
Set Width 1200
Set Height 400
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

Type "# AC-009: SAP-2 N/A marker test (RG-008) — no DTU; deferred D-2200"
Enter
Sleep 400ms

Type `cargo nextest run -p prism-sensors --test bc_2_16_016_claroty_ot_activity_events -E 'test(sap2)' 2>&1 | tail -8`
Enter
Sleep 30s
Sleep 2s
