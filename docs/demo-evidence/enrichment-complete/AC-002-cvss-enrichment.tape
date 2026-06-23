# AC-002 — Enrichment chain: cvss_base_score(device_cves_first) → 8.1 (HIGH)
#
# Demonstrates:
#   FROM armis_devices | where device_cves_first IS NOT NULL | enrich cvss_base_score(device_cves_first)
#   → cvss_base_score = 8.1, device_cves_first = CVE-9999-72859 (synthetic scenario CVE)
#
# Environment: PRISM_DTU_MODE=true, scenario Stage 4, client org-c
# Branch: fix/enrichment-complete HEAD d07025bf
# AC source: T13-capstone-demo-runbook.md §3 Step 3.5

Output docs/demo-evidence/enrichment-complete/AC-002-cvss-enrichment.gif
Output docs/demo-evidence/enrichment-complete/AC-002-cvss-enrichment.webm

Require python3

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
Set TypingSpeed 40ms

Hide
Sleep 1s
Show

Type "bash /tmp/run_ac002.sh"
Enter
Sleep 8s
