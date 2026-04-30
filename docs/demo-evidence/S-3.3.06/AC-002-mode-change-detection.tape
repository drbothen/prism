Output /Users/jmagady/Dev/prism/.worktrees/S-3.3.06/docs/demo-evidence/S-3.3.06/AC-002-mode-change-detection.gif
Output /Users/jmagady/Dev/prism/.worktrees/S-3.3.06/docs/demo-evidence/S-3.3.06/AC-002-mode-change-detection.webm

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

Type "cargo test -p prism-spec-engine --test bc_3_2_005_reload_mode_rejection test_BC_3_2_005_mode_change_detected_and_returned -- --nocapture 2>&1"
Enter
Sleep 30s
