Output AC-001-filter-basic-comparison.gif
Output AC-001-filter-basic-comparison.webm

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

Type "# AC-1: Filter mode — basic comparison (BC-2.11.002)"
Enter
Sleep 500ms

Type "# SUCCESS PATH: severity_id >= 3 → Predicate::Compare(Ge, 3)"
Enter
Sleep 500ms

Type "cargo test -p prism-query --test parser_tests test_AC_01_filter_basic_gte_comparison_produces_filter_expr test_AC_01_parse_filter_direct_produces_filter_expr test_BC_2_11_002_canonical_tv_severity_eq_critical -- --nocapture 2>&1 | tail -10"
Enter
Sleep 30s

Type "# ERROR PATH: missing predicate after pipe → Err(ParseError)"
Enter
Sleep 500ms

Type "cargo test -p prism-query --test parser_tests test_AC_09_malformed_filter_returns_structured_parse_error test_AC_09_parse_error_has_non_empty_message -- --nocapture 2>&1 | tail -10"
Enter
Sleep 30s
