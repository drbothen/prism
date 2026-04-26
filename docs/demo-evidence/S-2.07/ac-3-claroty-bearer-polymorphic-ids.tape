Output docs/demo-evidence/S-2.07/ac-3-claroty-bearer-polymorphic-ids.gif

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1000
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-2.07-per-sensor-auth"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-4: Claroty bearer auth + polymorphic IDs (BC-2.01.007)"
Enter
Type "cargo test -p prism-sensors --test test_claroty 2>&1 | tail -18"
Enter
Sleep 8s
Sleep 2s
