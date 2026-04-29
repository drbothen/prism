Output docs/demo-evidence/S-3.6.01/AC-001-hs-006-anchor-tests-green.gif
Output docs/demo-evidence/S-3.6.01/AC-001-hs-006-anchor-tests-green.webm

Require cargo

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set Framerate 30
Set PlaybackSpeed 1.0

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.6.01"
Enter
Sleep 500ms
Show

Type "cargo test --test hs_006_anchor_test 2>&1"
Enter
Sleep 12s
