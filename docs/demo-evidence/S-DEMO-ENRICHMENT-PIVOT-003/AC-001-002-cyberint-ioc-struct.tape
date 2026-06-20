# S-DEMO-ENRICHMENT-PIVOT-003 — AC-001 + AC-002
# Demonstrates: Cyberint Alert struct adds real-schema IOC fields (ioc, iocs[], alert_data)
# with serde dual-alias; fixture generator stamps scenario IOCs onto alert records.
#
# Evidence form: test-execution capture (cargo nextest)
# Tests:
#   test_BC_2_06_019_cyberint_alert_struct_has_real_ioc_fields       (AC-001)
#   test_BC_2_06_019_cyberint_ioc_struct_dual_alias_deserializes_both_key_forms  (AC-001)
#   test_BC_2_06_019_cyberint_fixture_generator_stamps_scenario_iocs  (AC-002)

Output docs/demo-evidence/S-DEMO-ENRICHMENT-PIVOT-003/AC-001-002-cyberint-ioc-struct.gif
Output docs/demo-evidence/S-DEMO-ENRICHMENT-PIVOT-003/AC-001-002-cyberint-ioc-struct.webm

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

# AC-001: Alert struct has real IOC fields + Ioc struct dual-alias (tolerates type/ioc_type)
Type "cargo nextest run -p prism-dtu-cyberint --features dtu,fixture-gen -E 'test(BC_2_06_019_cyberint_alert_struct)|test(BC_2_06_019_cyberint_ioc_struct)|test(BC_2_06_019_cyberint_fixture_generator)'"
Enter
Sleep 8s

Sleep 2s
