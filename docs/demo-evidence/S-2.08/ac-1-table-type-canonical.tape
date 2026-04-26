Output docs/demo-evidence/S-2.08/ac-1-table-type-canonical.gif

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

Type "# AC-1: TableType canonical definition in prism-core (BC-2.08)"
Enter
Type "cargo test -p prism-core table_type 2>&1 | grep -E 'test BC_2_08_table_type|test result.*ok'"
Enter
Sleep 12s
Sleep 2s
