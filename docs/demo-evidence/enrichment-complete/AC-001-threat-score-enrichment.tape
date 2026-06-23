# AC-001 — Enrichment chain: threat_score(iocs_value) → 95 (Malicious)
#
# Demonstrates:
#   FROM cyberint_alerts | where iocs_value IS NOT NULL | enrich threat_score(iocs_value)
#   → threat_score = 95, malicious = True (Malicious, threshold ≥ 75)
#
# Environment: PRISM_DTU_MODE=true, scenario Stage 4, client org-c
# Branch: fix/enrichment-complete HEAD d07025bf
# AC source: T13-capstone-demo-runbook.md §3 Step 3.2

Output docs/demo-evidence/enrichment-complete/AC-001-threat-score-enrichment.gif
Output docs/demo-evidence/enrichment-complete/AC-001-threat-score-enrichment.webm

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

Type "bash /tmp/run_ac001.sh"
Enter
Sleep 8s
