# VHS tape: AC-007, AC-015, AC-017 evidence
# Story: S-DEMO-DTU-LIVE-SCENARIO-001-A Baseline Seeding Retrofit
# Demonstrates: archetype-driven output — dormant=0 records, compromised=non-empty
# AC coverage: AC-007 (INV-FIXTURE-SET-ARCHETYPE-MAP-001), AC-015 (dormant empty), AC-017 (differential)
# Red Gate tests 15, 16, 17 in prism-dtu-demo-server

Output AC-007-015-017-archetype-differential.gif
Output AC-007-015-017-archetype-differential.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1280
Set Height 720
Set Theme "Catppuccin Mocha"
Set Padding 16
Set PlaybackSpeed 1.0

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-DTU-LIVE-SCENARIO-001-A"
Enter
Sleep 500ms
Show

Type "# AC-007/015/016/017: archetype reaches generate() — dormant=empty, large_scale=10000"
Enter
Sleep 400ms
Type "cargo nextest run -p prism-dtu-demo-server --features 'fixture-gen dtu' -E 'test(BC_2_06_018)' -q 2>&1"
Enter
Sleep 10000ms
