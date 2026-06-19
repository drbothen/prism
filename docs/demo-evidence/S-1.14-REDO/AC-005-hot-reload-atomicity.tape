Output AC-005-hot-reload-atomicity.gif
Output AC-005-hot-reload-atomicity.webm
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
Type "# AC-5: InfusionRegistry hot reload atomicity — CI-002 invariant (BC-2.19.004)"
Enter
Sleep 500ms
Type "cargo nextest run -p prism-spec-engine -E 'test(test_BC_2_19_004_valid_hot_reload_swaps_registry_atomically)' 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 4s
Type "# Error: failed reload retains prior registry (no corruption on bad spec)"
Enter
Sleep 300ms
Type "cargo nextest run -p prism-spec-engine -E 'test(test_BC_2_19_004_failed_hot_reload_retains_previous_registry)' 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 4s
Sleep 2s
