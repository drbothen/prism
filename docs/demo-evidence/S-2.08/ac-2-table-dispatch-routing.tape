Output docs/demo-evidence/S-2.08/ac-2-table-dispatch-routing.gif

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1000
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-2.08-event-tables"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-2 & AC-3: Table dispatch routing — EventStream buffer scan vs PointInTime live fetch"
Enter
Type "cargo test -p prism-sensors table_dispatch 2>&1 | grep -E 'test BC_2_08_route|test result.*ok'"
Enter
Sleep 12s
Sleep 2s
