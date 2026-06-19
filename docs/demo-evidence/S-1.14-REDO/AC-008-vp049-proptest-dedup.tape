Output AC-008-vp049-proptest-dedup.gif
Output AC-008-vp049-proptest-dedup.webm
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
Type "# AC-8: VP-049 proptest — for N values with K distinct, enrich_single called K times"
Enter
Sleep 500ms
Type "cargo nextest run -p prism-spec-engine -E 'test(dedup_proofs)' 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 6s
Type "# VP-049 unit analog: 500 events, 30 unique IPs → exactly 30 source calls"
Enter
Sleep 300ms
Type "cargo nextest run -p prism-spec-engine -E 'test(test_ac_10_vp_049_dedup_source_calls_equal_unique_value_count)' 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 4s
Sleep 2s
