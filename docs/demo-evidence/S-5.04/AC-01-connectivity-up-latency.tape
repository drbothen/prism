# AC-1: HTTP 200 -> status=Up + latency_ms non-zero (BC-2.08.001 postcondition 1)
# MockAdapterOk returns empty Ok(vec![]) simulating a successful 200 probe.
# check_one surfaces latency_ms on consumer-facing SensorHealthResult (F-S504-P1-MED-001).

Output docs/demo-evidence/S-5.04/AC-01-connectivity-up-latency.gif
Output docs/demo-evidence/S-5.04/AC-01-connectivity-up-latency.webm

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

Type "cargo nextest run -p prism-mcp -E 'test(live_probe_200_returns_up_with_latency) + test(check_one_surfaces_latency_ms)' --no-fail-fast 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 30s

Sleep 2s
