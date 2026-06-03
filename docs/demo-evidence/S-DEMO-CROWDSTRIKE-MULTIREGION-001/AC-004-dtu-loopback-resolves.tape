# AC-004 — CROWDSTRIKE_BASE_URL → DTU loopback address resolves in spec (unit path)
#
# Story:    S-DEMO-CROWDSTRIKE-MULTIREGION-001 v1.4
# AC:       AC-004 (BC-2.16.013 §Postconditions §2; ADR-031 §D8-c EC-005)
# Note:     Unit-level spec-load test only. Full DTU pipeline is #[ignore] per SID-1
#           (DTU-EXT-001: requires prism-dtu-crowdstrike; ungated after S-6.07).

Output docs/demo-evidence/S-DEMO-CROWDSTRIKE-MULTIREGION-001/AC-004-dtu-loopback-resolves.gif
Output docs/demo-evidence/S-DEMO-CROWDSTRIKE-MULTIREGION-001/AC-004-dtu-loopback-resolves.webm

Require cargo

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 18
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 50ms

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-CROWDSTRIKE-MULTIREGION-001 && clear"
Enter
Sleep 1s
Show

Type "cargo nextest run -p prism-spec-engine -E test(BC_2_16_013_crowdstrike_base_url_env_points_to_local_dtu_demo_works) 2>&1"
Enter
Sleep 600s
