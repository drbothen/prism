Output AC-010-hollow-feature-boot-wiring.gif
Output AC-010-hollow-feature-boot-wiring.webm
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
Type "# AC-10: Boot wires InfusionRegistry into QueryEngine — not just unit-tested in isolation"
Enter
Sleep 500ms
Type "cargo nextest run -p prism-bin -E 'test(test_boot_with_csv_infusion_udf_resolves)' 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 5s
Type "# End-to-end: boot + query executes UDF against CSV fixture without silent drop"
Enter
Sleep 300ms
Type "cargo nextest run -p prism-bin -E 'test(test_boot_with_csv_infusion_udf_query_resolves)' 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 5s
Sleep 2s
