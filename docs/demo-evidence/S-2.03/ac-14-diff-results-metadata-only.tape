Output ac-14-diff-results-metadata-only.gif

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

Type "# AC-14: diff_results metadata-only cols; no previous_results; EC-005 unknown table=None (BC-2.15.011)"
Enter
Type "cargo test -p prism-storage --lib -- test_BC_2_15_011_diff_results_columns_metadata_only test_BC_2_15_011_ec005_get_descriptor_unknown_table_returns_none --nocapture 2>&1 | tail -8"
Enter
Sleep 20s
Sleep 2s
