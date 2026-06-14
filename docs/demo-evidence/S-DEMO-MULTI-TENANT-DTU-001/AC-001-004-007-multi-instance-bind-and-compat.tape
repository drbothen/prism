# S-DEMO-MULTI-TENANT-DTU-001 — AC-001/AC-002/AC-003/AC-004/AC-007
# Demonstrates: multi-instance bind config accepted (AC-001), two armis on distinct ports (AC-002),
# two claroty on distinct ports (AC-003), harness per-org socket map (AC-004),
# backward compatibility (AC-007). Tests complete in ~6s.

Output docs/demo-evidence/S-DEMO-MULTI-TENANT-DTU-001/AC-001-004-007-multi-instance-bind-and-compat.gif
Output docs/demo-evidence/S-DEMO-MULTI-TENANT-DTU-001/AC-001-004-007-multi-instance-bind-and-compat.webm

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

Type "# AC-001/002/003/004/007: bind config + distinct ports + compat"
Enter
Sleep 500ms

Type "cargo nextest run -p prism-dtu-demo-server -p prism-dtu-harness --all-features -E 'test(BC_2_06_017_demo_server_multi_instance_bind_config_accepted) + test(BC_2_06_017_demo_server_two_armis) + test(BC_2_06_017_demo_server_two_claroty) + test(BC_2_06_017_harness_multi_instance_builds) + test(BC_2_06_017_harness_distinct_org) + test(BC_2_06_017_single_instance)' --no-fail-fast 2>&1 | grep -E 'PASS|FAIL|Summary|Starting'"
Enter
Sleep 12s

Sleep 2s
