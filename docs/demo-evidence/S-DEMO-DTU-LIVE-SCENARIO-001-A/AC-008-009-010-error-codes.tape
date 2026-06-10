# VHS tape: AC-008, AC-009, AC-010 evidence
# Story: S-DEMO-DTU-LIVE-SCENARIO-001-A Baseline Seeding Retrofit
# Demonstrates: E-DEMO-001 (bad fixture_set), E-DEMO-004 (missing org_id), E-DEMO-005 (invalid UUID)
# AC coverage: AC-008 (INV-CONSTRUCTION-TIME-FAILURE-001), AC-009 (E-DEMO-004), AC-010 (E-DEMO-005)

Output AC-008-009-010-error-codes.gif
Output AC-008-009-010-error-codes.webm

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

Type "# AC-008/009/010: E-DEMO-001/004/005 at construction time (INV-CONSTRUCTION-TIME-FAILURE-001)"
Enter
Sleep 400ms
Type "cargo nextest run -p prism-dtu-demo-server --features 'fixture-gen dtu' -E 'test(e_demo)' -q 2>&1"
Enter
Sleep 6000ms
