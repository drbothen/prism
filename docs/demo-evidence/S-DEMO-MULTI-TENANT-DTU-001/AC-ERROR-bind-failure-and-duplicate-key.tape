# S-DEMO-MULTI-TENANT-DTU-001 — Error Paths
# Demonstrates: bind failure aggregates all errors (EC-001 / BC-2.06.017 Postcondition 6),
# duplicate instance name rejected (EC-009 / Postcondition 7, demo-server),
# duplicate harness key rejected (EC-003 / Postcondition 7, harness),
# zero instances returns empty map (EC-002).
# Tests complete in < 1s.

Output docs/demo-evidence/S-DEMO-MULTI-TENANT-DTU-001/AC-ERROR-bind-failure-and-duplicate-key.gif
Output docs/demo-evidence/S-DEMO-MULTI-TENANT-DTU-001/AC-ERROR-bind-failure-and-duplicate-key.webm

Require cargo

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
Set TypingSpeed 40ms

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-MULTI-TENANT-DTU-001"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# Error paths: bind failure aggregation + duplicate key/name + empty config"
Enter
Sleep 500ms

Type "cargo nextest run -p prism-dtu-demo-server -p prism-dtu-harness --all-features -E 'test(BC_2_06_017_demo_server_bind_failure) + test(BC_2_06_017_demo_server_duplicate) + test(BC_2_06_017_demo_server_zero) + test(BC_2_06_017_harness_duplicate) + test(BC_2_06_017_harness_bind_failure)' --no-fail-fast 2>&1 | grep -E 'PASS|FAIL|Summary|Starting'"
Enter
Sleep 5s

Sleep 2s
