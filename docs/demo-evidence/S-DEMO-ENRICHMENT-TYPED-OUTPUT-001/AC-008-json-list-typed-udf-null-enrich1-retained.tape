# AC-008 — JSON-list input to typed-output UDF produces NULL + E-INFUSE-014;
#           json-typed ENRICH-1 list-dispatch path retained
#
# Demonstrates:
#   RGT-010: test_json_list_input_to_typed_output_udf_produces_null_e_infuse_014 PASS
#           input "[\"hash1\",\"hash2\"]" + output_type="integer" → NULL (not ENRICH-1 dispatch)
#           input "[\"hash1\",\"hash2\"]" + output_type="json" → ENRICH-1 dispatch RETAINED
#
# BC: BC-2.19.001 v2.2 EC-19-008; INV-ENRICH-TYPED-001 clause 4; TV-19-001-json-list-typed; ADR-051 D4
# Crate: prism-query
# Branch: feature/S-DEMO-ENRICHMENT-TYPED-OUTPUT-001 HEAD 622bd2fa

Output docs/demo-evidence/S-DEMO-ENRICHMENT-TYPED-OUTPUT-001/AC-008-json-list-typed-udf-null-enrich1-retained.gif
Output docs/demo-evidence/S-DEMO-ENRICHMENT-TYPED-OUTPUT-001/AC-008-json-list-typed-udf-null-enrich1-retained.webm

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
Type "cargo nextest run -p prism-query -E 'test(json_list_input_to_typed_output_udf_produces_null)' 2>/dev/null"
Enter
Sleep 10s
Show

Type "# AC-008: JSON-list '[...]' to typed UDF → NULL (prevents double-encoding; ENRICH-1 only for json-typed)"
Enter
Sleep 300ms
Type "cargo nextest run -p prism-query -E 'test(json_list_input_to_typed_output_udf_produces_null)'"
Enter
Sleep 4s
