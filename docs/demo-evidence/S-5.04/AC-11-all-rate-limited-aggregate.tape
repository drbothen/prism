# AC-11: all sensors rate-limited -> overall_status="rate_limited" (BC-2.08.007 EC-08-015)
# Verbatim summary: "0 of N sensors healthy for client 'X' -- all rate-limited".
# Error path: mixed rate-limited + down -> Partial (not RateLimited).

Output docs/demo-evidence/S-5.04/AC-11-all-rate-limited-aggregate.gif
Output docs/demo-evidence/S-5.04/AC-11-all-rate-limited-aggregate.webm

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

Type "cargo nextest run -p prism-mcp -E 'test(EC_007_all_rate_limited_aggregate_yields_rate_limited) + test(EC_007_mixed_rate_limited_and_down_is_partial)' --no-fail-fast 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 30s

Sleep 2s
