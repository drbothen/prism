Output AC-009-no-unimplemented-stubs.gif
Output AC-009-no-unimplemented-stubs.webm
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
Type "# AC-9: No unimplemented!()/todo!() in production infusion modules (POL-12)"
Enter
Sleep 500ms
Type "cargo nextest run -p prism-bin -E 'test(pol_12)' 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 5s
Type "# VP-048: N distinct fields → N descriptors (correctness proof)"
Enter
Sleep 300ms
Type "cargo nextest run -p prism-spec-engine -E 'test(test_ac_9_vp_048_n_distinct_fields_n_descriptors)' 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 4s
Sleep 2s
