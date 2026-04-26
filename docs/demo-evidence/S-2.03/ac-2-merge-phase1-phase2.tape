Output ac-2-merge-phase1-phase2.gif

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

Type "# AC-2: merge carries Phase 1+2 fields; analyst_id and query_source set (BC-2.15.010)"
Enter
Type "cargo test -p prism-storage --lib -- test_BC_2_15_010_merge_without_periodic_carries_phase1_and_phase2 --nocapture 2>&1 | tail -8"
Enter
Sleep 20s
Sleep 2s
