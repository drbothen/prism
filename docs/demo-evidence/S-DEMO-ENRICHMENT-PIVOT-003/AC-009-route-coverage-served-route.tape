# S-DEMO-ENRICHMENT-PIVOT-003 — AC-009
# Demonstrates: BC-2.06.019 Route Coverage Table StageMask served-route guards all ACTIVE
# Armis device_cves, CrowdStrike ioc_hashes, Cyberint ioc_* route guards confirmed working.
#
# Evidence form: test-execution capture (cargo nextest)
# Tests:
#   test_BC_2_06_019_served_route_to_enrich_pipeline_composed_full_chain           (AC-009)
#   test_BC_2_06_019_armis_served_route_to_nvd_enrich_pipeline_composed_full_chain (AC-009)
#   test_BC_2_06_019_crowdstrike_ioc_bearing_detection_stagemask_served_route      (AC-009)
#   test_BC_2_06_019_cyberint_alerts_stagemask_ioc_filter                          (AC-009)

Output docs/demo-evidence/S-DEMO-ENRICHMENT-PIVOT-003/AC-009-route-coverage-served-route.gif
Output docs/demo-evidence/S-DEMO-ENRICHMENT-PIVOT-003/AC-009-route-coverage-served-route.webm

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

# AC-009: Route Coverage Table served-route guards (Rows 6, 8-11) all ACTIVE
Type "cargo nextest run -p prism-dtu-demo-server -p prism-dtu-crowdstrike -p prism-dtu-cyberint --features dtu,fixture-gen -E 'test(BC_2_06_019_served_route)|test(BC_2_06_019_armis_served_route)|test(BC_2_06_019_crowdstrike_ioc_bearing)|test(BC_2_06_019_cyberint_alerts_stagemask_ioc_filter)'"
Enter
Sleep 28s

Sleep 2s
