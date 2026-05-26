Output AC-001-overlay-discovery-and-merge.gif
Output AC-001-overlay-discovery-and-merge.webm

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

Type "# AC-001: Overlay file discovery and scalar merge (BC-2.06.012)"
Enter
Sleep 500ms

Type "# SUCCESS PATH: customers/acme/armis.sensor.toml overlay sets base_url"
Enter
Sleep 300ms
Type "# Asserts: ResolvedSensorSpec(acme, armis).base_url == overlay URL, INV-OVL-001 tables, provenance"
Enter
Sleep 500ms

Type "cargo nextest run -p prism-spec-engine -E 'test(test_BC_2_06_012_overlay_discovered_and_merged)' 2>&1 | tail -8"
Enter
Sleep 10s
