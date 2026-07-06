# AC-006 — InfusionLoader::validate rejects plugin-type field without source_column (E-INFUSE-013 sub-cond 8)
# AC-007 — InfusionLoader::validate rejects unknown output_type (E-INFUSE-013 sub-cond 7)
#
# Demonstrates:
#   RGT-002: test_plugin_type_field_without_source_column_rejected_e_infuse_013 PASS
#   RGT-011: test_unknown_output_type_rejected_e_infuse_013_sub_condition_7 PASS
#   Spec rejected at load time — not at query time; other infusions unaffected
#
# BC: BC-2.19.001 v2.2 E-INFUSE-013 sub-cond 7/8; EC-19-009; ADR-051 D3
# Crate: prism-spec-engine
# Branch: feature/S-DEMO-ENRICHMENT-TYPED-OUTPUT-001 HEAD 622bd2fa

Output docs/demo-evidence/S-DEMO-ENRICHMENT-TYPED-OUTPUT-001/AC-006-007-loader-validation-e-infuse-013.gif
Output docs/demo-evidence/S-DEMO-ENRICHMENT-TYPED-OUTPUT-001/AC-006-007-loader-validation-e-infuse-013.webm

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
Type "cargo nextest run -p prism-spec-engine -E 'test(rejected_e_infuse_013)' 2>/dev/null"
Enter
Sleep 10s
Show

Type "# AC-006/007: E-INFUSE-013 spec-load rejection (sub-cond 7: unknown output_type; sub-cond 8: missing source_column)"
Enter
Sleep 300ms
Type "cargo nextest run -p prism-spec-engine -E 'test(rejected_e_infuse_013)'"
Enter
Sleep 4s
