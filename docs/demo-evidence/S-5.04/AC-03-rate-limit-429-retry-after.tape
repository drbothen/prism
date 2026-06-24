# AC-3: HTTP 429 + Retry-After: 30 -> is_rate_limited=true, retry_after ~30s (BC-2.08.003)
# EC-003 error path: 429 with no header -> default 60s backoff.
# 429 on live probe yields ConnectivityStatus::Up (sensor is reachable) + rate_limit populated.

Output docs/demo-evidence/S-5.04/AC-03-rate-limit-429-retry-after.gif
Output docs/demo-evidence/S-5.04/AC-03-rate-limit-429-retry-after.webm

Require cargo

Set Shell "zsh"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 20
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 50ms

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-5.04"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "cargo nextest run -p prism-mcp -E 'test(rate_limit_state_from_http_429_with_header) + test(rate_limit_default_60s_when_no_header) + test(live_probe_429_yields_up_and_populates_rate_limit)' --no-fail-fast 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 30s

Sleep 2s
