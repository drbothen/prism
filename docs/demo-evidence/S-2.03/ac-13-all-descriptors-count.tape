Output ac-13-all-descriptors-count.gif

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

Type "# AC-13: all_descriptors() returns exactly 7 entries (BC-2.15.011)"
Enter
Type "cargo test -p prism-storage --lib -- test_BC_2_15_011_all_descriptors_count_and_names --nocapture 2>&1 | tail -8"
Enter
Sleep 20s
Sleep 2s
