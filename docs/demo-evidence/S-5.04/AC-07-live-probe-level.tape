# AC-7: check_sensor_health returns probe_level="live", reachable/auth_valid are booleans (BC-2.08.005)
# Error path: spec-only path returns probe_level="spec-only".

Output docs/demo-evidence/S-5.04/AC-07-live-probe-level.gif
Output docs/demo-evidence/S-5.04/AC-07-live-probe-level.webm

Require cargo

Set Shell "zsh"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 20
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 50ms

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-5.04"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "cargo nextest run -p prism-mcp -E 'test(S504_live_probe_sets_probe_level_live) + test(sensors_health_resource_live_probe_level) + test(sensors_health_resource_reachable_is_boolean) + test(check_sensor_health_returns_spec_only_probe_level)' --no-fail-fast 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 30s

Sleep 2s
