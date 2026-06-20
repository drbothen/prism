# S-DEMO-ENRICHMENT-PIVOT-003 — AC-003
# Demonstrates: _ioc_value synthetic filter REMOVED; real-schema filter ADDED (atomic)
# Verifies grep returns 0 matches for _ioc_value in alerts.rs; real-schema filter tests pass.
#
# Evidence form: test-execution capture (cargo nextest)
# Tests:
#   test_BC_2_06_019_cyberint_alerts_real_schema_ioc_filter_no_synthetic  (AC-003)
#   test_BC_2_06_019_ioc_hashes_false_withholds_cyberint_alert_with_matching_hash  (AC-003 / Red Gate 10)

Output docs/demo-evidence/S-DEMO-ENRICHMENT-PIVOT-003/AC-003-real-schema-filter.gif
Output docs/demo-evidence/S-DEMO-ENRICHMENT-PIVOT-003/AC-003-real-schema-filter.webm

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

# Verify synthetic _ioc_value filter is absent (grep returns 0 matches)
Type "grep -c '_ioc_value' crates/prism-dtu-cyberint/src/routes/alerts.rs && echo 'FAIL: synthetic present' || echo 'PASS: 0 matches - synthetic filter removed'"
Enter
Sleep 3s

# AC-003: Real-schema filter tests — route withholds IOC-matching alerts
Type "cargo nextest run -p prism-dtu-cyberint --features dtu,fixture-gen -E 'test(BC_2_06_019_cyberint_alerts_real_schema_ioc_filter)|test(BC_2_06_019_ioc_hashes_false_withholds)'"
Enter
Sleep 12s

Sleep 2s
