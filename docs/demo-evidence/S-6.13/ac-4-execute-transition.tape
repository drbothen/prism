Output ac-4-execute-transition.gif

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

Type "# AC-4: POST transition id 11 -> 204; status becomes In Progress"
Enter
Type "cargo test -p prism-dtu-jira --features dtu --test fidelity test_inprogress_to_done_transition_id_21 --nocapture 2>&1 | grep -E 'test_inprogress_to_done|PASSED|FAILED|ok$|error\['"
Enter
Sleep 90s
