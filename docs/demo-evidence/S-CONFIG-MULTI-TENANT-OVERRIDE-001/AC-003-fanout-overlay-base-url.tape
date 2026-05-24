Output AC-003-fanout-overlay-base-url.gif
Output AC-003-fanout-overlay-base-url.webm

Require cargo

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set Padding 20
Set FontFamily "FiraCode Nerd Font Mono"

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-CONFIG-MULTI-TENANT-OVERRIDE-001"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-003: Instance identity resolution at fanout uses overlay base_url (BC-2.06.014)"
Enter
Sleep 500ms

Type "# Case A: overlay base_url used at HTTP dispatch"
Enter
Sleep 300ms
Type "# Case B: no overlay -> TYPE spec base_url (backwards compat)"
Enter
Sleep 300ms
Type "# F-LP2-CRIT-001: Arc-DI fanout routes HTTP mock to per-org endpoint (end-to-end)"
Enter
Sleep 500ms

Type "cargo nextest run -p prism-spec-engine -E 'test(test_BC_2_06_014_resolved_spec_overlays_base_url)' 2>&1 | tail -8"
Enter
Sleep 10s

Type "cargo nextest run -p prism-sensors -E 'test(test_F_LP2_CRIT_001_fan_out_with_overlay_map_routes_to_overlay_url)' 2>&1 | tail -8"
Enter
Sleep 12s
