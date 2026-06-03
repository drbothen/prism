# AC-002 — eu-1 URL resolves from CROWDSTRIKE_BASE_URL env var (happy path)
#
# Story:    S-DEMO-CROWDSTRIKE-MULTIREGION-001 v1.4
# AC:       AC-002 (BC-2.16.013 §Postconditions §1; BC-2.16.009 EC-009-008)
# Evidence: Red Gate test passes: CROWDSTRIKE_BASE_URL=eu-1 URL; spec.base_url resolves.

Output docs/demo-evidence/S-DEMO-CROWDSTRIKE-MULTIREGION-001/AC-002-eu1-base-url-resolves.gif
Output docs/demo-evidence/S-DEMO-CROWDSTRIKE-MULTIREGION-001/AC-002-eu1-base-url-resolves.webm

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

Type "cargo nextest run -p prism-spec-engine -E test(BC_2_16_013_crowdstrike_eu1_base_url_env_var_resolves_correctly) 2>&1"
Enter
Sleep 600s
