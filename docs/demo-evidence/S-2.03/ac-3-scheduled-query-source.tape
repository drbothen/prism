Output ac-3-scheduled-query-source.gif

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1000
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-2.03-decorators-internal-tables"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-3: scheduled query => analyst_id=None, query_source=schedule:{name} (BC-2.15.009 EC-15-034)"
Enter
Type "cargo test -p prism-storage --lib -- test_BC_2_15_009_scheduled_query_analyst_id_none_query_source_schedule --nocapture 2>&1 | tail -8"
Enter
Sleep 20s
Sleep 2s
