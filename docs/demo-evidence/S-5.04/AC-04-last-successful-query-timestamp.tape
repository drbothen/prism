# AC-4: last_successful_query("acme", CrowdStrike) returns the written timestamp (BC-2.08.004)
# write_timestamp + read_timestamp round-trip proves per-key storage.
# Error path: read before write returns None.

Output docs/demo-evidence/S-5.04/AC-04-last-successful-query-timestamp.gif
Output docs/demo-evidence/S-5.04/AC-04-last-successful-query-timestamp.webm

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

Type "cargo nextest run -p prism-mcp -E 'test(write_and_read_timestamp_roundtrip) + test(checker_record_and_read_timestamp) + test(read_timestamp_returns_none_before_write)' --no-fail-fast 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 30s

Sleep 2s
