Output ac-9-rate-limit-429.gif

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1000
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-6.12-dtu-pagerduty"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-9: FailureMode::RateLimit -> HTTP 429 + Retry-After: 60"
Enter
Type "cargo test -p prism-dtu-pagerduty --features dtu --test fidelity test_ac9_rate_limit_returns_429_with_retry_after --nocapture 2>&1 | grep -E 'ac9|PASSED|FAILED|ok$'"
Enter
Sleep 60s
