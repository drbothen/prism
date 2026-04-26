Output ac-1-config-time-phase1.gif

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

Type "# AC-1: get_config_time returns Phase 1 fields; Phase 2/3 absent (BC-2.15.010)"
Enter
Type "cargo test -p prism-storage --lib -- test_BC_2_15_010_get_config_time_phase1_fields_populated --nocapture 2>&1 | tail -8"
Enter
Sleep 20s
Sleep 2s
