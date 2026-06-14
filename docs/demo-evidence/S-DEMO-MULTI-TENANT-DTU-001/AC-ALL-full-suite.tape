# S-DEMO-MULTI-TENANT-DTU-001 — Full test suite (all 19 tests)
# Demonstrates: all 19 multi-instance tests passing across prism-dtu-demo-server
# and prism-dtu-harness, covering AC-001 through AC-007 success + error paths.
# Tests complete in ~6s (includes two 5s backward-compat shutdown tests).

Output docs/demo-evidence/S-DEMO-MULTI-TENANT-DTU-001/AC-ALL-full-suite.gif
Output docs/demo-evidence/S-DEMO-MULTI-TENANT-DTU-001/AC-ALL-full-suite.webm

Require cargo

Set Shell "zsh"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 16
Set Width 1200
Set Height 680
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

Type "# S-DEMO-MULTI-TENANT-DTU-001: full 19-test suite (BC-2.06.017)"
Enter
Sleep 500ms

Type "cargo nextest run -p prism-dtu-demo-server -p prism-dtu-harness --all-features -E 'binary(multi_instance_tests) + binary(multi_instance_harness_tests)' --no-fail-fast 2>&1 | grep -E 'PASS|FAIL|Summary|Starting'"
Enter
Sleep 12s

Sleep 2s
