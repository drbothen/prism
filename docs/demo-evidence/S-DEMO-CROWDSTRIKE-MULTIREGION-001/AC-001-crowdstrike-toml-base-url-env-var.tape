# AC-001 — crowdstrike.sensor.toml has base_url = "${env.CROWDSTRIKE_BASE_URL}"
#
# Story:    S-DEMO-CROWDSTRIKE-MULTIREGION-001 v1.4
# AC:       AC-001 (BC-2.16.013 §Postconditions §1)
# Evidence: Shows the production TOML with env-var base_url and 4-region comment block.
#           Also confirms D-747 LOCKED auth_type and auth_plugin are unchanged.

Output docs/demo-evidence/S-DEMO-CROWDSTRIKE-MULTIREGION-001/AC-001-crowdstrike-toml-base-url-env-var.gif
Output docs/demo-evidence/S-DEMO-CROWDSTRIKE-MULTIREGION-001/AC-001-crowdstrike-toml-base-url-env-var.webm

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

# Show region runbook comment + env-var base_url (success path)
Type "grep -A10 region crates/prism-sensors/specs/crowdstrike.sensor.toml | head -14"
Enter
Sleep 2s

# Confirm hardcoded us-1 URL is absent (error path guard)
Type "grep -c api.crowdstrike.com crates/prism-sensors/specs/crowdstrike.sensor.toml || echo absent"
Enter
Sleep 1.5s

Sleep 2s
