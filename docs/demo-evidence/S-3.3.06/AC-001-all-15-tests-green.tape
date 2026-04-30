Output /Users/jmagady/Dev/prism/.worktrees/S-3.3.06/docs/demo-evidence/S-3.3.06/AC-001-all-15-tests-green.gif
Output /Users/jmagady/Dev/prism/.worktrees/S-3.3.06/docs/demo-evidence/S-3.3.06/AC-001-all-15-tests-green.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1280
Set Height 720
Set Theme "Catppuccin Mocha"
Set Padding 24
Set Shell "zsh"

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.3.06"
Enter
Sleep 1s
Show

Type "cargo test -p prism-spec-engine --test bc_3_2_005_reload_mode_rejection 2>&1 | tail -25"
Enter
Sleep 30s
