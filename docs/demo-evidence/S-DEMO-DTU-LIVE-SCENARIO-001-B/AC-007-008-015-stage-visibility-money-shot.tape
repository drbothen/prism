# AC-007, AC-008, AC-015
# THE MONEY SHOT — Unfolding-attack live demo: stage visibility + cross-DTU coherence
#
# Covers:
#   AC-007 — Armis: primary_device absent at stage 0 (T+30s), present at stage 1+ (T+90s);
#              lateral devices absent at stage 1
#   AC-008 — CrowdStrike: containment_status="contained" only at stage 4 (T+700s)
#   AC-015 — Cross-DTU entity coherence: same "dev-deadbeef-100-0" ID present across
#              Armis, CrowdStrike, and Claroty at stage 1 (T+90s)
#
# Traces to: BC-2.06.019 postcondition 4 (TV-019-009, TV-019-010, TV-019-011),
#             BC-2.06.020 INV-CROSS-DTU-ENTITY-COHERENCE-001 (postcondition 5)
# Story: S-DEMO-DTU-LIVE-SCENARIO-001-B

Output AC-007-008-015-stage-visibility-money-shot.gif
Output AC-007-008-015-stage-visibility-money-shot.webm

Require cargo

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 13
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set Framerate 30
Set PlaybackSpeed 1

Hide
Type "export PS1='$ '"
Enter
Wait+Line /\$/
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-DTU-LIVE-SCENARIO-001-B"
Enter
Wait+Line /\$/
Show

Type "# AC-007: Armis primary device absent stage-0 (T+30s), visible stage-1+ (T+90s)"
Enter
Sleep 500ms
Type "cargo nextest run -p prism-dtu-armis --features 'dtu,fixture-gen' -E 'test(BC_2_06_019_armis_primary_device_stage_visibility)' 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 16s
Wait+Line /\$/
Sleep 1s

Type "# AC-008: CrowdStrike containment_status='contained' only at stage-4 (T+700s)"
Enter
Sleep 500ms
Type "cargo nextest run -p prism-dtu-crowdstrike --features 'dtu,fixture-gen' -E 'test(BC_2_06_019_crowdstrike_containment_visible_at_stage4_only)' 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 16s
Wait+Line /\$/
Sleep 1s

Type "# AC-015: Cross-DTU coherence — 'dev-deadbeef-100-0' consistent across Armis+CS+Claroty"
Enter
Sleep 500ms
Type "cargo nextest run -p prism-dtu-demo-server --features fixture-gen -E 'test(BC_2_06_020_cross_dtu_entity_coherence_stage1_all_three_clones)' 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 20s
Wait+Line /\$/
Sleep 2s
