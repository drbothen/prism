# AC-6: 3 up + 1 down -> OverallStatus::Partial (success response, not PrismError) (BC-2.08.007)
# Invariant: Partial is not an error. All auth-invalid -> Unhealthy (not Partial).

Output docs/demo-evidence/S-5.04/AC-06-partial-aggregate-3up-1down.gif
Output docs/demo-evidence/S-5.04/AC-06-partial-aggregate-3up-1down.webm

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

Type "cargo nextest run -p prism-mcp -E 'test(aggregate_partial_when_some_up_some_down) + test(aggregate_healthy_when_all_up) + test(aggregate_unhealthy_when_all_down) + test(invariant_partial_is_not_error) + test(aggregate_all_auth_invalid_is_unhealthy)' --no-fail-fast 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 30s

Sleep 2s
