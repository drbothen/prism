Output ac-4-rate-limit-429.gif

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1000
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-6.11-dtu-slack"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-4: FailureMode::RateLimit after 3 requests -> HTTP 429 + Retry-After: 30 + body ratelimited"
Enter
Type "cargo test -p prism-dtu-slack --features dtu --test ac_tests ac_4_rate_limit_returns_429_with_retry_after_and_ratelimited_body --nocapture 2>&1 | grep -E 'ac_4|PASSED|FAILED|ok$|error\['"
Enter
Sleep 60s
