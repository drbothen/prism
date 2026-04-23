Output docs/demo-evidence/S-1.05/AC-004-cyberint-bad-timestamp-error.gif
Output docs/demo-evidence/S-1.05/AC-004-cyberint-bad-timestamp-error.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Catppuccin Mocha"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 24

# AC-4 (BC-2.02.004, BC-2.02.011): Cyberint "not-a-date" → Err(OcsfTimestampParseError)
#                                    with field name and raw value in error.

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-1.05-ocsf-field-mapping"
Enter
Sleep 500ms
Type "clear"
Enter
Sleep 300ms
Show

Type "# AC-4 (error path): malformed created_date='not-a-date' → OcsfTimestampParseError"
Enter
Sleep 800ms
Type "cargo test -p prism-ocsf test_BC_2_02_004_cyberint_malformed_timestamp_returns_error 2>&1 | grep -E 'test.*ok|test.*FAILED|test result'"
Enter
Sleep 15s
