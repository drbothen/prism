Output docs/demo-evidence/S-2.08/ac-10-inject-source-type-edge-cases.gif

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

Type "# AC-10: inject_source_type — PointInTime and cold-start rows get _source_type=live"
Enter
Type "cargo test -p prism-query inject_source_type 2>&1 | grep -E 'test BC_2_08_inject_source_type_(point_in_time|cold_start|non_object|operates_on)|test result.*ok'"
Enter
Sleep 12s
Sleep 2s
