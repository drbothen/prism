Output ac-5-add-comment.gif

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

Type "# AC-5: POST transition invalid id from Done -> 400 + 'Invalid transition id'"
Enter
Type "cargo test -p prism-dtu-jira --features dtu --test fidelity test_inprogress_to_inprogress_is_invalid_transition --nocapture 2>&1 | grep -E 'test_inprogress_to_inprogress|PASSED|FAILED|ok$|error\['"
Enter
Sleep 90s
