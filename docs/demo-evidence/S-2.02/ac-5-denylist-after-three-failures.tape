Output ac-5-denylist-after-three-failures.gif

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1000
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-2.02-audit-buffer-watchdog"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-5: 3 failures -> denylisted; submission -> Err(QueryDenylisted) (BC-2.15.008)"
Enter
Type "cargo test -p prism-storage --lib -- test_BC_2_15_008_three test_BC_2_15_008_query test_BC_2_15_008_third test_BC_2_15_008_intervening test_denylist_expiry --nocapture 2>&1 | tail -10"
Enter
Sleep 15s
