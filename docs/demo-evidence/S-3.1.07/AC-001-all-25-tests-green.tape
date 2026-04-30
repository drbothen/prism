Output AC-001-all-25-tests-green.gif
Output AC-001-all-25-tests-green.webm

Require cargo

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Dracula"
Set Padding 24
Set WaitTimeout 120s

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.1.07"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "cargo test -p prism-audit 2>&1 | tail -30"
Enter
Sleep 30s
Sleep 2s
