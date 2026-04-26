Output docs/demo-evidence/S-2.07/ac-5-pagination-offset-cursor.gif

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

Type "# AC-5: Offset-based hybrid pagination, 5 requests for 500 entries (BC-2.01.004)"
Enter
Type "cargo test -p prism-sensors --test test_pagination 2>&1 | tail -17"
Enter
Sleep 8s
Sleep 2s
