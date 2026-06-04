# S-DEMO-002 / AC-003 + AC-004 + AC-005 + AC-006
# AC-003: CrowdStrike query returns non-empty Arrow batches with OCSF fields (detection_id, category_uid, class_uid).
# AC-004: Armis query WHERE aql='in:devices' returns non-empty Arrow batches.
# AC-005: Claroty alerts (alert_type_name, detected_time) + devices (uid) return data.
# AC-006: Cyberint alerts return non-empty Arrow batches.
#
# BC: BC-2.11.005 postcondition — sensor responses normalized to OCSF.
# Red Gate tests: test_BC_2_11_005_e2e_crowdstrike_query_returns_ocsf_data,
#                 test_BC_2_11_005_e2e_armis_query_returns_data,
#                 test_BC_2_11_005_e2e_claroty_query_returns_data,
#                 test_BC_2_11_005_e2e_cyberint_query_returns_data

Output "docs/demo-evidence/S-DEMO-002/AC-003-006-four-sensor-data-return.gif"
Output "docs/demo-evidence/S-DEMO-002/AC-003-006-four-sensor-data-return.webm"

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 18
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 40ms

Sleep 2s

Hide
Type "export PATH=/Users/jmagady/.cargo/bin:$PATH"
Enter
Sleep 500ms
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-002"
Enter
Sleep 500ms
Show

# All 4 sensor E2E tests: CrowdStrike + Armis + Claroty + Cyberint
Type "# AC-003..006: all 4 sensors return OCSF data rows via DTU subprocess round-trip"
Enter
Sleep 500ms
Type "cargo nextest run -p prism-bin --profile e2e --run-ignored all -E 'test(e2e_crowdstrike_query) | test(e2e_armis_query) | test(e2e_claroty_query) | test(e2e_cyberint_query)' 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 30s

Sleep 2s
