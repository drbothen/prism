# AC-013, AC-014, AC-016
# Enrichment Correlation — ThreatIntel IOC injection + NVD CVE injection + passthrough
#
# Covers:
#   AC-013 — ThreatIntelClone::new_with_scenario: all scenario IOCs (IPs, domains, hashes)
#              resolve as known_malicious=true, threat_score>=75
#   AC-014 — NvdClone::new_with_scenario: scenario CVEs resolve with
#              cvss_metric_v31[0].cvss_data.base_score>=7.0, base_severity="HIGH"
#   AC-016 — Non-scenario IOC passthrough is strictly additive; perimeter gate passes
#              (neither threatintel nor nvd imports spec-engine/sensors/query)
#
# Traces to: BC-2.06.020 INV-THREATINTEL-IOC-CORRELATION-001 (postconditions 1,2),
#             BC-2.06.020 INV-NVD-CVE-CORRELATION-001 (postconditions 3,4),
#             BC-2.06.020 INV-NON-SCENARIO-LOOKUP-PASSTHROUGH-001,
#             BC-2.06.020 INV-PERIMETER-COMPLIANCE-001
# Story: S-DEMO-DTU-LIVE-SCENARIO-001-B

Output AC-013-014-016-enrichment-correlation.gif
Output AC-013-014-016-enrichment-correlation.webm

Require cargo

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 13
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set Framerate 30
Set PlaybackSpeed 1

Hide
Type "export PS1='$ '"
Enter
Wait+Line /\$/
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-DTU-LIVE-SCENARIO-001-B"
Enter
Wait+Line /\$/
Show

Type "# AC-013+016: ThreatIntel IOC injection (score>=75) + non-scenario passthrough + perimeter"
Enter
Sleep 500ms
Type "cargo nextest run -p prism-dtu-threatintel --features 'dtu,fixture-gen' -E 'test(BC_2_06_020)' 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 12s
Wait+Line /\$/
Sleep 1s

Type "# AC-014: NVD CVE correlation — base_score>=7.0, base_severity='HIGH'"
Enter
Sleep 500ms
Type "cargo nextest run -p prism-dtu-nvd --features 'dtu,fixture-gen' -E 'test(BC_2_06_020)' 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 28s
Wait+Line /\$/
Sleep 2s
