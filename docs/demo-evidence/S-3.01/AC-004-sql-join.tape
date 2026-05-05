Output AC-004-sql-join.gif
Output AC-004-sql-join.webm

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

Type "# AC-4: SQL mode — INNER/LEFT/RIGHT/FULL JOIN (BC-2.11.003)"
Enter
Sleep 500ms

Type "# SUCCESS PATH: all JOIN kinds produce correct AST nodes"
Enter
Sleep 500ms

Type "cargo test -p prism-query --test parser_tests test_AC_04 test_BC_2_11_003_left_join_kind_parsed test_BC_2_11_003_right_join_kind_parsed test_BC_2_11_003_full_outer_join_kind_parsed -- --nocapture 2>&1 | tail -12"
Enter
Sleep 30s
