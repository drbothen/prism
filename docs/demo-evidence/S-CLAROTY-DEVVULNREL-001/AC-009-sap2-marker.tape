# AC-009 (SAP-2 N/A marker test)
#
# SAP-2 DTU-parity probe is N/A for this story per BC-2.16.017 §PC5 and D-2200.
# No DTU exists for the device_vulnerability_relations endpoint; near-term tests
# run against the live monroe sensor only. DTU creation is deferred to a separate
# story (xdome-endpoint-expansion-plan.md §Deferred DTU-Creation Stories).
#
# This test asserts the SAP2_STATUS constant to document that adversarial review
# MUST NOT file SAP-2 parity findings against this story delivery.
#
# Tests: SAP-2 N/A marker (test_BC_2_16_017_...sap2_na_documented) — prism-sensors
#        SAP-2 N/A marker (test_BC_2_16_017_...wire_shape_sap2_na_documented) — prism-bin
# Traces to: BC-2.16.017 §Postconditions §5 (SAP-2 N/A)

Output "docs/demo-evidence/S-CLAROTY-DEVVULNREL-001/AC-009-sap2-marker.gif"
Output "docs/demo-evidence/S-CLAROTY-DEVVULNREL-001/AC-009-sap2-marker.webm"

Require cargo

Set Shell "zsh"
Set WaitTimeout 180s
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
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-CLAROTY-DEVVULNREL-001"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 0.5s
Show

Type "# AC-009 SAP-2 N/A marker — no DTU for device_vulnerability_relations (deferred D-2200)"
Enter
Sleep 400ms

Type `cargo nextest run -p prism-sensors -p prism-bin -E 'test(sap2_na_documented)' --no-fail-fast 2>&1 | tail -15`
Enter
Sleep 120s
Sleep 2s
