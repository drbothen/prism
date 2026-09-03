# AC-WIRE-001: wire-level proof — serialized POST bodies contain sort_by
# Covers all 7 ACs: proves build_request does NOT clobber sort_by during
# offset/limit injection. Uses wiremock mock server (no live network, no tenant data).
# Tests: test_obs1_vulnerabilities_build_request_emits_sort_by
#        test_obs1_audit_logs_build_request_emits_sort_by_with_filter_and_pagination
#        test_obs3_remaining_tables_build_request_emits_sort_by

Output AC-WIRE-001-obs-wire-sort_by-serialized.gif
Output AC-WIRE-001-obs-wire-sort_by-serialized.webm

Set Shell "zsh"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 18
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 50ms
Set WaitTimeout 120s

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/DEFECT-CLAROTY-SORTBY-DETERMINISM-001"
Enter
Sleep 0.5s
Type "cargo build -p prism-sensors --tests 2>/dev/null"
Enter
Sleep 30s
Type "clear"
Enter
Sleep 0.3s
Show

Type "cargo nextest run -p prism-sensors -E 'test(obs)' --no-fail-fast 2>&1 | tail -12"
Enter
Wait+Screen /Summary/
Sleep 1s

Sleep 2s
