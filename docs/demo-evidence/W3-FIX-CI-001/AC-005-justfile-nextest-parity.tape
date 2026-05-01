Output AC-005-justfile-nextest-parity.gif
Output AC-005-justfile-nextest-parity.webm

Require just

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Catppuccin Mocha"
Set Padding 20
Set Shell "zsh"

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/W3-FIX-CI-001"
Enter
Sleep 1s
Show

Type "just --show check"
Enter
Sleep 3s

Type "just --show check-ci"
Enter
Sleep 3s
