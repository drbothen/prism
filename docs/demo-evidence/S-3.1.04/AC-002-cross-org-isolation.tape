Output AC-002-cross-org-isolation.gif
Output AC-002-cross-org-isolation.webm

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

Type "cargo test -p prism-credentials --test bc_3_2_002_org_id_namespace test_BC_3_2_002_tv_02_cross_org_isolation -- --nocapture"
Enter
Sleep 30s
