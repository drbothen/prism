Output docs/demo-evidence/S-2.08/ac-8-fanout-dispatch.gif

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

Type "# AC-8 & AC-7: TableSpec TOML parsing + TableType importable from prism-core"
Enter
Type "cargo test -p prism-spec-engine --test bc_2_16_table_type_test 2>&1 | grep -E 'test BC_2_08_table_spec|test BC_2_08_table_type|test BC_2_08_toml|test result.*ok'"
Enter
Sleep 12s
Sleep 2s
