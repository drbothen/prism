# AC-009 — threatintel.infusion.toml: source_column on all fields, iocs_value_first as input_field
#           Fixes double-encoding defect ADV-P11-OBS-001
#
# Demonstrates:
#   RGT-012: test_threatintel_toml_has_source_column_and_iocs_value_first_input_field PASS
#   RGT-023: test_threat_sources_json_output_no_double_encoding PASS
#           (scalar iocs_value_first input + source_column → ["greynoise","abuseipdb"] single-encoded)
#
# BC: BC-2.19.001 v2.2 Plugin-type field projection postcondition; INV-ENRICH-TYPED-001 clause 3
# Crates: prism-spec-engine, prism-query
# Branch: feature/S-DEMO-ENRICHMENT-TYPED-OUTPUT-001 HEAD 622bd2fa

Output docs/demo-evidence/S-DEMO-ENRICHMENT-TYPED-OUTPUT-001/AC-009-threatintel-toml-no-double-encoding.gif
Output docs/demo-evidence/S-DEMO-ENRICHMENT-TYPED-OUTPUT-001/AC-009-threatintel-toml-no-double-encoding.webm

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
Type "cargo nextest run -p prism-spec-engine -E 'test(threatintel_toml)' -p prism-query -E 'test(threat_sources_json_output_no_double_encoding)' 2>/dev/null"
Enter
Sleep 10s
Show

Type "# AC-009: threatintel TOML source_column + iocs_value_first (kills double-encoding)"
Enter
Sleep 300ms
Type "cargo nextest run -p prism-spec-engine -E 'test(threatintel_toml)'"
Enter
Sleep 4s

Type "cargo nextest run -p prism-query -E 'test(threat_sources_json_output_no_double_encoding)'"
Enter
Sleep 4s
