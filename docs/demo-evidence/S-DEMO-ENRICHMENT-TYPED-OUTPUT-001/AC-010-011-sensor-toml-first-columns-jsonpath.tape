# AC-010 — Scalar _first companion columns in cyberint/crowdstrike sensor TOMLs
# AC-011 — _first columns populated via JSONPath source_path from DTU nested arrays
#
# Demonstrates:
#   RGT-013: test_cyberint_sensor_toml_has_iocs_value_first_column PASS
#   RGT-014: test_crowdstrike_sensor_toml_has_behaviors_ioc_value_first_column PASS
#   RGT-015: test_ac011_cyberint_alerts_iocs_value_first_column_via_jsonpath PASS
#           ($.iocs[0].value JSONPath + generator stamps iocs[0].value)
#   RGT-016: test_ac011_crowdstrike_detections_behaviors_ioc_value_first_column_via_jsonpath PASS
#           ($.behaviors[0].ioc_value; top-level scalar ABSENT confirmed)
#
# BC: BC-2.19.001 v2.2 INV-ENRICH-TYPED-001 clause 4; ADR-051 D4; SAP-2
# Crates: prism-spec-engine, prism-dtu-cyberint (fixture-gen), prism-dtu-crowdstrike (fixture-gen)
# Branch: feature/S-DEMO-ENRICHMENT-TYPED-OUTPUT-001 HEAD 622bd2fa

Output docs/demo-evidence/S-DEMO-ENRICHMENT-TYPED-OUTPUT-001/AC-010-011-sensor-toml-first-columns-jsonpath.gif
Output docs/demo-evidence/S-DEMO-ENRICHMENT-TYPED-OUTPUT-001/AC-010-011-sensor-toml-first-columns-jsonpath.webm

Set Shell "zsh"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 16
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 40ms

Hide
Type "export PATH=/Users/jmagady/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin"
Enter
Sleep 200ms
Type "cargo nextest run -p prism-spec-engine -E 'test(sensor_toml_has)' 2>/dev/null"
Enter
Sleep 10s
Show

Type "# AC-010: _first companion columns declared in sensor TOMLs"
Enter
Sleep 300ms
Type "cargo nextest run -p prism-spec-engine -E 'test(sensor_toml_has)'"
Enter
Sleep 4s

Type "# AC-011: JSONPath source_path extraction in DTU generators (fixture-gen feature)"
Enter
Sleep 300ms
Type "cargo nextest run -p prism-dtu-cyberint --features fixture-gen -E 'test(ac011)'"
Enter
Sleep 4s

Type "cargo nextest run -p prism-dtu-crowdstrike --features fixture-gen -E 'test(ac011)'"
Enter
Sleep 4s
