Output AC-002-filter-advanced-predicates.gif
Output AC-002-filter-advanced-predicates.webm

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

Type "# AC-2: Filter mode — AND/OR/NOT, HAS/MISSING, IN, LIKE, CIDR (BC-2.11.002)"
Enter
Sleep 500ms

Type "# SUCCESS PATH: AND combinator + advanced predicates green"
Enter
Sleep 500ms

Type "cargo test -p prism-query --test parser_tests test_AC_02 test_BC_2_11_002 -- --nocapture 2>&1 | tail -15"
Enter
Sleep 30s

Type "# ERROR PATH: invalid regex pattern rejected (CWE-1333)"
Enter
Sleep 500ms

Type "cargo test -p prism-query test_literal_regex_validated_at_parse_time_invalid_rejected test_literal_regex_size_capped_at_1024_bytes -- --nocapture 2>&1 | tail -10"
Enter
Sleep 30s
