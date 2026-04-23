Output docs/demo-evidence/S-1.05/AC-003-cyberint-unix-timestamp.gif
Output docs/demo-evidence/S-1.05/AC-003-cyberint-unix-timestamp.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Catppuccin Mocha"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 24

# AC-3 (BC-2.02.004): Cyberint created_date Unix integer 1710498600 → 2024-03-15T10:30:00Z.

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-1.05-ocsf-field-mapping"
Enter
Sleep 500ms
Type "clear"
Enter
Sleep 300ms
Show

Type "# AC-3: Cyberint Unix timestamp 1710498600 parses to 2024-03-15T10:30:00Z"
Enter
Sleep 800ms
Type "cargo test -p prism-ocsf test_BC_2_02_004_cyberint_unix_timestamp_parsed_correctly test_BC_2_02_004_cyberint_rfc3339_timestamp_parsed_correctly test_BC_2_02_004_cyberint_iso8601_no_tz_parsed_correctly 2>&1 | grep -E 'test.*ok|test.*FAILED|test result'"
Enter
Sleep 15s
