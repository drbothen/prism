Output ac-2-get-issue.gif

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

Type "# AC-2: POST comment -> 201; GET issue shows comment.total incremented"
Enter
Type "cargo test -p prism-dtu-jira --features dtu --test fidelity test_get_issue_response_has_self_field_and_status_id --nocapture 2>&1 | grep -E 'test_get_issue|PASSED|FAILED|ok$|error\['"
Enter
Sleep 90s
