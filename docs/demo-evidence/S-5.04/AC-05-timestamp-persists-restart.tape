# AC-5: Timestamp survives Prism restart (checker reconstruction on shared PrismContext) (BC-2.08.004 postcondition 2)
# Second SensorHealthChecker built on same PrismContext reads prior timestamp.

Output docs/demo-evidence/S-5.04/AC-05-timestamp-persists-restart.gif
Output docs/demo-evidence/S-5.04/AC-05-timestamp-persists-restart.webm

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

Type "cargo nextest run -p prism-mcp -E 'test(timestamp_survives_checker_reconstruction) + test(timestamp_survives_context_reconstruction_with_storage) + test(checker_last_successful_query_none_before_record)' --no-fail-fast 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 30s

Sleep 2s
