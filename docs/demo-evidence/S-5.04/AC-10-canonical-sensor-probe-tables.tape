# AC-10: All 4 canonical sensor TOMLs declare probe_table matching a real [[tables]] block (BC-2.08.001 + BC-2.16.009)
# crowdstrike->detections, cyberint->alerts, claroty->devices, armis->devices.

Output docs/demo-evidence/S-5.04/AC-10-canonical-sensor-probe-tables.gif
Output docs/demo-evidence/S-5.04/AC-10-canonical-sensor-probe-tables.webm

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

Type "cargo nextest run -p prism-spec-engine -E 'test(canonical_sensor_toml_probe_table_declared)' --no-fail-fast 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 60s

Sleep 2s
