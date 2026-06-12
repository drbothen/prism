# AC-001 through AC-006, AC-012
# Scenario Progression Core — IncidentTimeline types, pure function, stage boundaries,
# monotonicity, clock-skew, StageMask completeness, secondary RNG independence
#
# Covers: AC-001 (timeline types + non_exhaustive), AC-002 (pure function),
#         AC-003 (stage boundaries: 5 thresholds, 6 TV vectors),
#         AC-004 (monotonicity), AC-005 (clock skew clamped to 0),
#         AC-006 (StageMask 6 fields all explicit), AC-012 (RNG independence)
# Traces to: BC-2.06.019 precondition 3, INV-PROGRESSION-REPRODUCIBILITY-001,
#             postconditions 2/3, INV-STAGE-MONOTONICITY-001, EC-019-003,
#             INV-STAGE-MASK-COMPLETENESS-001, INV-SECONDARY-RNG-STREAM-INDEPENDENCE-001
# Story: S-DEMO-DTU-LIVE-SCENARIO-001-B

Output AC-001-006-012-scenario-progression-core.gif
Output AC-001-006-012-scenario-progression-core.webm

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

Type "# AC-001..006, AC-012: timeline types, pure stage fn, stage boundaries, masks, RNG independence"
Enter
Sleep 500ms

Type "cargo nextest run -p prism-dtu-common --features fixture-gen -E 'test(BC_2_06_019)' --no-fail-fast 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 6s
Wait+Line /\$/
Sleep 2s
