Output ac-6-store-periodic-failure-stale-value.gif

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

Type "# AC-6: store_periodic failure => warn log; stale value returned (BC-2.15.010 E-DECOR-001)"
Enter
Type "cargo test -p prism-storage --lib -- test_BC_2_15_010_store_periodic_failure_stale_value_pattern --nocapture 2>&1 | tail -8"
Enter
Sleep 20s
Sleep 2s
