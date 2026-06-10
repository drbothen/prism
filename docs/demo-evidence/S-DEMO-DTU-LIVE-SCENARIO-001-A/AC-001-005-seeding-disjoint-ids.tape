# VHS tape: AC-001, AC-002, AC-005 evidence
# Story: S-DEMO-DTU-LIVE-SCENARIO-001-A Baseline Seeding Retrofit
# Demonstrates: org_slug derivation + INV-DISTINCT-DATA-001 (disjoint device IDs)
# AC coverage: AC-001 (ScenarioEntityCatalog), AC-002 (org_slug format), AC-005 (disjoint IDs)

Output AC-001-005-seeding-disjoint-ids.gif
Output AC-001-005-seeding-disjoint-ids.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1280
Set Height 720
Set Theme "Catppuccin Mocha"
Set Padding 16
Set PlaybackSpeed 1.0

# Navigate (hidden)
Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-DTU-LIVE-SCENARIO-001-A"
Enter
Sleep 500ms
Show

Type "# INV-DISTINCT-DATA-001: Client A (seed=100/deadbeef) vs B (seed=200/cafebabe)"
Enter
Sleep 400ms
Type "cargo run -p prism-dtu-crowdstrike --features fixture-gen --example demo_seeding -q"
Enter
Sleep 6000ms
