Output ac-9-unknown-issue-key-404.gif

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

Type "# AC-9: GET /issue/UNKNOWN-999 -> 404 'Issue does not exist'"
Enter
Type "cargo test -p prism-dtu-jira --features dtu --test fidelity test_unknown_issue_key_returns_404 test_execute_transition_on_missing_issue_returns_404 test_list_transitions_on_missing_issue_returns_404 test_add_comment_on_missing_issue_returns_404 --nocapture 2>&1 | grep -E 'test_unknown_issue|test_execute_trans|test_list_trans|test_add_comment_on|PASSED|FAILED|ok$|error\['"
Enter
Sleep 90s
