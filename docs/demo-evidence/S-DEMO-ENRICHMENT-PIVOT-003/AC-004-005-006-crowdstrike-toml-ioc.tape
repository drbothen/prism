# S-DEMO-ENRICHMENT-PIVOT-003 — AC-004 + AC-005 + AC-006
# AC-004: CrowdStrike detection generator stamps behaviors[].ioc_type/ioc_value in JSON records
# AC-005: CrowdStrike detections TOML spec declares behaviors[] IOC columns
# AC-006: Cyberint sensor TOML spec declares ioc, iocs[], alert_data.* columns
#
# Evidence form: test-execution capture (cargo nextest)
# Tests:
#   test_BC_2_06_019_crowdstrike_detection_behaviors_ioc_hash_stamped   (AC-004)
#   test_BC_2_06_019_crowdstrike_detection_behaviors_no_ioc_when_none   (AC-004 error path)
#   test_BC_2_06_019_crowdstrike_generator_scenario_parity_modulo_ioc_stamp (AC-004)
#   test_BC_2_06_019_crowdstrike_detection_toml_spec_has_ioc_columns    (AC-005)
#   test_BC_2_06_019_cyberint_alert_toml_spec_has_ioc_columns           (AC-006)

Output docs/demo-evidence/S-DEMO-ENRICHMENT-PIVOT-003/AC-004-005-006-crowdstrike-toml-ioc.gif
Output docs/demo-evidence/S-DEMO-ENRICHMENT-PIVOT-003/AC-004-005-006-crowdstrike-toml-ioc.webm

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

# AC-004/005/006: CrowdStrike behaviors[] IOC stamping + TOML spec columns
Type "cargo nextest run -p prism-dtu-crowdstrike -p prism-dtu-cyberint --features dtu,fixture-gen -E 'test(BC_2_06_019_crowdstrike_detection_behaviors)|test(BC_2_06_019_crowdstrike_detection_toml)|test(BC_2_06_019_cyberint_alert_toml)|test(BC_2_06_019_crowdstrike_generator_scenario_parity)'"
Enter
Sleep 8s

Sleep 2s
