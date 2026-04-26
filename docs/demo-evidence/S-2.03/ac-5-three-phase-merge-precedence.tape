Output ac-5-three-phase-merge-precedence.gif

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

Type "# AC-5: three-phase merge precedence periodic>query-time>config-time (BC-2.15.010)"
Enter
Type "cargo test -p prism-storage --lib -- test_BC_2_15_010_merge_precedence_periodic_wins_over_query_time_and_config_time test_BC_2_15_010_ec001_merge_with_none_periodic_sensor_health_absent --nocapture 2>&1 | tail -8"
Enter
Sleep 20s
Sleep 2s
