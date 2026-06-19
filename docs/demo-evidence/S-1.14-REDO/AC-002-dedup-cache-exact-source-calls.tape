Output AC-002-dedup-cache-exact-source-calls.gif
Output AC-002-dedup-cache-exact-source-calls.webm
Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 16
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 30ms
Hide
Sleep 2s
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-1.14-REDO && clear"
Enter
Sleep 1s
Show
Type "# AC-2: 500 events, 30 unique IPs → enrich_single called exactly 30 times (BC-2.19.002)"
Enter
Sleep 500ms
Type "cargo nextest run -p prism-spec-engine -E 'test(test_BC_2_19_002_ten_thousand_rows_two_hundred_unique_ips_two_hundred_calls)' 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 4s
Type "# Isolation invariant: per-query caches do not share state"
Enter
Sleep 300ms
Type "cargo nextest run -p prism-spec-engine -E 'test(test_BC_2_19_002_invariant_per_query_cache_is_isolated)' 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 4s
Sleep 2s
