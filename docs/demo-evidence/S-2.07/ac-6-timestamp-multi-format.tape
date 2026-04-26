Output docs/demo-evidence/S-2.07/ac-6-timestamp-multi-format.gif

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

Type "# AC-3 (timestamp): Multi-format parsing: RFC 3339, Unix epoch, custom (BC-2.01.006)"
Enter
Type "cargo test -p prism-sensors --test test_timestamp 2>&1 | tail -16"
Enter
Sleep 8s
Sleep 2s
