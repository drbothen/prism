Output ac-10-rate-limit-429.gif

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

Type "# AC-10: FailureMode::RateLimit -> 429; issue NOT persisted (EC-006 atomicity)"
Enter
Type "cargo test -p prism-dtu-jira --features dtu --test fidelity test_ac10_rate_limit_429_returned_and_issue_not_persisted --nocapture 2>&1 | grep -E 'test_ac10|PASSED|FAILED|ok$|error\['"
Enter
Sleep 90s
