Output ac-7-vp-058-proptest-grace-period.gif

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

Type "# AC-7: VP-058 proptest — terminate iff consecutive_over_limit >= 2 (BC-2.15.007)"
Enter
Type "cargo test -p prism-storage --lib -- test_BC_2_15_007_VP058 --nocapture 2>&1 | tail -10"
Enter
Sleep 20s
