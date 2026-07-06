# AC-004 — coerce_to_typed() failure: NULL output + E-INFUSE-014 tracing::warn!
# AC-005 — InfusionError::TypeCoercionFailed variant with E-INFUSE-014 message format
#
# Demonstrates:
#   RGT-007: test_coerce_to_typed_integer_failure_produces_null_e_infuse_014 PASS
#   RGT-008: test_coerce_to_typed_float_failure_produces_null_e_infuse_014 PASS
#   RGT-009: test_coerce_to_typed_boolean_unrecognized_value_produces_null_e_infuse_014 PASS
#   Also: RGT-017..020 positive coerce_to_typed paths; RGT-021/022 EC-002/006 edge cases
#
# BC: BC-2.19.001 v2.2 E-INFUSE-014; INV-ENRICH-TYPED-001 clause 5; ADR-051 D2
# Crate: prism-query
# Branch: feature/S-DEMO-ENRICHMENT-TYPED-OUTPUT-001 HEAD 622bd2fa

Output docs/demo-evidence/S-DEMO-ENRICHMENT-TYPED-OUTPUT-001/AC-004-005-coercion-failure-null-e-infuse-014.gif
Output docs/demo-evidence/S-DEMO-ENRICHMENT-TYPED-OUTPUT-001/AC-004-005-coercion-failure-null-e-infuse-014.webm

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
Type "cargo nextest run -p prism-query -E 'test(coerce_to_typed)' 2>/dev/null"
Enter
Sleep 10s
Show

Type "# AC-004/005: coercion failure → NULL row + E-INFUSE-014 (not panic)"
Enter
Sleep 300ms
Type "cargo nextest run -p prism-query -E 'test(coerce_to_typed)'"
Enter
Sleep 4s
