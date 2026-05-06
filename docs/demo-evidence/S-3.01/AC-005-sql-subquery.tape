Output AC-005-sql-subquery.gif
Output AC-005-sql-subquery.webm

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

Type "# AC-5: SQL mode — subquery in WHERE IN (BC-2.11.003)"
Enter
Sleep 500ms

Type "# SUCCESS PATH: SELECT ... WHERE field IN (SELECT ...) → Predicate::InSubquery"
Enter
Sleep 500ms

Type "cargo test -p prism-query --test parser_tests test_AC_05 -- --nocapture 2>&1 | tail -10"
Enter
Sleep 30s
