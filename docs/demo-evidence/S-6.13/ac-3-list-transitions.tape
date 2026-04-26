Output ac-3-list-transitions.gif

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1000
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-6.13-dtu-jira"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-3: GET /transitions when Open -> ids 11 (Start Progress) and 31 (Close) present"
Enter
Type "cargo test -p prism-dtu-jira --features dtu --test fidelity test_open_transition_names_are_start_progress_and_close --nocapture 2>&1 | grep -E 'test_open_transition|PASSED|FAILED|ok$|error\['"
Enter
Sleep 90s
