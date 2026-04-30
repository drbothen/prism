Output AC-001-all-24-tests-green.gif
Output AC-001-all-24-tests-green.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Catppuccin Mocha"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 20
Set PlaybackSpeed 1.0

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.1.04"
Enter
Sleep 1s
Show

Type "cargo test -p prism-credentials --test bc_3_2_002_org_id_namespace 2>&1 | tail -30"
Enter
Sleep 30s
