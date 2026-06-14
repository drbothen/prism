# S-DEMO-MULTI-TENANT-DTU-001 — AC-005/AC-006
# Demonstrates: per-org base_url overlay loads distinct ResolvedSensorSpec (AC-005);
# zero cross-tenant leakage — org A reaches only instance A, org B reaches only instance B
# (AC-006, server-side AtomicU64 counter proof via /dtu/request-count endpoint).
# Tests complete in < 1s.

Output docs/demo-evidence/S-DEMO-MULTI-TENANT-DTU-001/AC-005-006-overlay-and-isolation-proof.gif
Output docs/demo-evidence/S-DEMO-MULTI-TENANT-DTU-001/AC-005-006-overlay-and-isolation-proof.webm

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

Type "# AC-005: distinct base_url per org  |  AC-006: zero cross-tenant leakage"
Enter
Sleep 500ms

Type "cargo nextest run -p prism-dtu-harness --all-features -E 'test(BC_2_06_017_multi_instance_overlay) + test(BC_2_06_017_multi_tenant)' --no-fail-fast 2>&1 | grep -E 'PASS|FAIL|Summary|Starting'"
Enter
Sleep 5s

Sleep 2s
