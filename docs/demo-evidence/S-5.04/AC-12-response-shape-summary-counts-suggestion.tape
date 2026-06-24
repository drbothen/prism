# AC-12: structuredContent shape: overall_status + summary_counts + per-sensor suggestion (BC-2.08.007)
# Verbatim suggestion strings verified for auth-invalid, unreachable, and rate-limited states.

Output docs/demo-evidence/S-5.04/AC-12-response-shape-summary-counts-suggestion.gif
Output docs/demo-evidence/S-5.04/AC-12-response-shape-summary-counts-suggestion.webm

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

Type "cargo nextest run -p prism-mcp -E 'test(EC_007_response_shape_overall_status_summary_counts_suggestion) + test(EC_08_015_auth_invalid_production_path_suggestion) + test(EC_08_015_genuine_down_production_path_suggestion)' --no-fail-fast 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 30s

Sleep 2s
