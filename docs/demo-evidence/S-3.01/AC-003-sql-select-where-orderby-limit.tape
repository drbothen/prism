Output AC-003-sql-select-where-orderby-limit.gif
Output AC-003-sql-select-where-orderby-limit.webm

Require cargo

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set Padding 20
Set FontFamily "FiraCode Nerd Font Mono"

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.01"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-3: SQL mode — SELECT/FROM/WHERE/ORDER BY/LIMIT (BC-2.11.003)"
Enter
Sleep 500ms

Type "# SUCCESS PATH: full SELECT query parses to correct AST"
Enter
Sleep 500ms

Type "cargo test -p prism-query --test parser_tests test_AC_03 test_BC_2_11_003_canonical_tv_select_with_group_by test_BC_2_11_003_select_distinct_modifier -- --nocapture 2>&1 | tail -15"
Enter
Sleep 30s

Type "# ERROR PATH: mutation statements (INSERT/UPDATE/DELETE) rejected"
Enter
Sleep 500ms

Type "cargo test -p prism-query --test parser_tests test_BC_2_11_003_canonical_tv_mutation_insert_rejected test_BC_2_11_003_mutation_update_rejected test_BC_2_11_003_mutation_delete_rejected -- --nocapture 2>&1 | tail -10"
Enter
Sleep 30s
