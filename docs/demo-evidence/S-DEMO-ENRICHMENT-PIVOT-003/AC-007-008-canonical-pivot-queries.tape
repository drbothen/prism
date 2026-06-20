# S-DEMO-ENRICHMENT-PIVOT-003 — AC-007 + AC-008
# AC-007: Canonical ThreatIntel pivot query returns Malicious results at stage >= 3
# AC-008: Canonical NVD pivot query returns HIGH CVSS results at stage 4 (Containment)
#
# Evidence form: test-execution capture (cargo nextest)
# Tests:
#   test_BC_2_06_019_canonical_threatintel_pivot_query_returns_malicious_at_stage_3  (AC-007)
#   test_BC_2_06_019_canonical_nvd_pivot_query_returns_high_cvss_at_containment_stage (AC-008)
#   test_BC_2_06_019_enrich_pipeline_e2e_threatintel_pivot_executes_udf_and_returns_malicious (AC-007)
#   test_BC_2_06_019_enrich_pipeline_e2e_nvd_pivot_executes_udf_and_returns_high_cvss  (AC-008)

Output docs/demo-evidence/S-DEMO-ENRICHMENT-PIVOT-003/AC-007-008-canonical-pivot-queries.gif
Output docs/demo-evidence/S-DEMO-ENRICHMENT-PIVOT-003/AC-007-008-canonical-pivot-queries.webm

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
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-ENRICHMENT-PIVOT-003"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

# AC-007: ThreatIntel pivot at stage >=3 returns threat_is_known_malicious=true, threat_score>=75
# AC-008: NVD pivot at stage 4 returns cvss_base_score>=7.0 (uses has device_cves_first filter)
Type "cargo nextest run -p prism-dtu-demo-server --features dtu,fixture-gen -E 'test(BC_2_06_019_canonical)|test(BC_2_06_019_enrich_pipeline_e2e)'"
Enter
Sleep 12s

Sleep 2s
