Output TAP-001-release-gate-suite.gif
Output TAP-001-release-gate-suite.webm

Require bash

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 13
Set Width 1200
Set Height 800
Set Theme "Dracula"
Set Padding 24
Set WaitTimeout 60s

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-REL-001"
Enter
Sleep 500ms
Type "clear"
Enter
Sleep 300ms
Show

Type "bash tests/release-gate/run.sh"
Enter
Sleep 30s
Sleep 2s
