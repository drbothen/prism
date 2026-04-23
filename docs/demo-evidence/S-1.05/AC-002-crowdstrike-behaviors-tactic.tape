Output docs/demo-evidence/S-1.05/AC-002-crowdstrike-behaviors-tactic.gif
Output docs/demo-evidence/S-1.05/AC-002-crowdstrike-behaviors-tactic.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Catppuccin Mocha"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 24

# AC-2 (BC-2.02.003): CrowdStrike behaviors[0].tactic → attacks[0].tactic.name.

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-1.05-ocsf-field-mapping"
Enter
Sleep 500ms
Type "clear"
Enter
Sleep 300ms
Show

Type "# AC-2: CrowdStrike behaviors[0].tactic='Discovery' → attacks[0].tactic.name"
Enter
Sleep 800ms
Type "cargo test -p prism-ocsf test_BC_2_02_003_crowdstrike_behaviors_tactic_maps_to_attacks 2>&1 | grep -E 'test.*ok|test.*FAILED|test result'"
Enter
Sleep 15s
