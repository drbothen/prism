Output docs/demo-evidence/S-2.08/ac-7-table-spec-validation.gif

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

Type "# AC-7: SpecParser rejects poll_interval < 10s and retention > 7d (EC-002)"
Enter
Type "cargo test -p prism-spec-engine --test bc_2_16_table_type_test 2>&1 | grep -E 'test BC_2_08_validate_table_spec|test result.*ok'"
Enter
Sleep 12s
Sleep 2s
