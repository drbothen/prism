Output docs/demo-evidence/S-2.07/ac-4-armis-aql-forwarding.gif

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

Type "# AC-6: Armis AQL forwarding + timestamp fallback chain (BC-2.01.008)"
Enter
Type "cargo test -p prism-sensors --test test_armis 2>&1 | tail -14"
Enter
Sleep 8s
Sleep 2s
