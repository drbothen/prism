Output ac-1-create-issue.gif

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

Type "# AC-1: POST /rest/api/3/issue -> HTTP 201, key PROJ-NNNN, /dtu/issues shows Open"
Enter
Type "cargo test -p prism-dtu-jira --features dtu --test fidelity test_full_lifecycle_create_comment_transition --nocapture 2>&1 | grep -E 'test_full_lifecycle|PASSED|FAILED|ok$|error\[|AC-1'"
Enter
Sleep 90s
