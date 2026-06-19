# S-DEMO-ENRICHMENT-PIVOT-002 — AC-005 + AC-006
# Demonstrates: | enrich threat_intel(ioc_value) returns Malicious for scenario IOCs
#               | enrich nvd(device_cves_first) returns HIGH CVSS for scenario CVEs
# Both use in-process mock sources — no live demo server needed.
#
# Evidence form: test-execution capture

Output docs/demo-evidence/S-DEMO-ENRICHMENT-PIVOT-002/AC-005-006-pipe-stage-enrich.gif
Output docs/demo-evidence/S-DEMO-ENRICHMENT-PIVOT-002/AC-005-006-pipe-stage-enrich.webm

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
Set TypingSpeed 50ms
Set WaitTimeout 120s

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-ENRICHMENT-PIVOT-002"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

# AC-005: enrich threat_intel pipe stage returns malicious for scenario IOCs
Type "cargo nextest run -p prism-spec-engine -E 'test(enrich_threatintel_pipe_stage_returns_malicious)'"
Enter
Sleep 15s

# AC-006: enrich nvd pipe stage returns HIGH CVSS for scenario CVEs
Type "cargo nextest run -p prism-spec-engine -E 'test(enrich_nvd_pipe_stage_returns_high_cvss)'"
Enter
Sleep 15s

Sleep 2s
