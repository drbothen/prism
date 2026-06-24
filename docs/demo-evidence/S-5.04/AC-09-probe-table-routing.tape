# AC-9: probe routes LIMIT 0 to probe_table table; fallback to first declared table (BC-2.08.001 postcondition 5)
# Error path: org_id not in spec_map -> sentinel "devices" fallback.

Output docs/demo-evidence/S-5.04/AC-09-probe-table-routing.gif
Output docs/demo-evidence/S-5.04/AC-09-probe-table-routing.webm

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

Type "cargo nextest run -p prism-mcp -E 'test(check_one_routes_to_probe_table_via_spec_map) + test(check_one_falls_back_to_first_table_via_spec_map) + test(check_one_falls_back_to_devices_when_org_not_in_spec_map)' --no-fail-fast 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 30s

Sleep 2s
