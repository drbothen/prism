Output ac-7-unknown-issuetype-400.gif

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

Type "# AC-7: POST issue unknown issuetype 'Feature' -> 400 {errors: {issuetype: unknown}}"
Enter
Type "cargo test -p prism-dtu-jira --features dtu --test fidelity test_unknown_issuetype_returns_400 --nocapture 2>&1 | grep -E 'test_unknown_issuetype|PASSED|FAILED|ok$|error\['"
Enter
Sleep 90s
