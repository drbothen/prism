# AC-001 — output_arrow_type() returns correct DataType for all 6 output_type values
# AC-002 — return_type() delegates to output_arrow_type(); ZERO hardcoded DataType::Utf8
#
# Demonstrates:
#   RGT-001: test_return_type_matches_output_type_for_all_declared_types PASS
#   AC-002: rg 'return_type.*Utf8' crates/prism-query/src/infusion_udf.rs → 0 matches
#
# BC: BC-2.19.001 v2.2 INV-ENRICH-TYPED-001 clause 1; ADR-051 D1
# Crate: prism-query
# Branch: feature/S-DEMO-ENRICHMENT-TYPED-OUTPUT-001 HEAD 622bd2fa

Output docs/demo-evidence/S-DEMO-ENRICHMENT-TYPED-OUTPUT-001/AC-001-002-output-arrow-type-no-hardcoded-utf8.gif
Output docs/demo-evidence/S-DEMO-ENRICHMENT-TYPED-OUTPUT-001/AC-001-002-output-arrow-type-no-hardcoded-utf8.webm

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
Type "cargo nextest run -p prism-query -E 'test(test_return_type_matches_output_type_for_all_declared_types)' 2>/dev/null"
Enter
Sleep 10s
Show

Type "# AC-001/002: typed UDF output — no hardcoded Utf8 in return_type()"
Enter
Sleep 300ms
Type "cargo nextest run -p prism-query -E 'test(test_return_type_matches_output_type_for_all_declared_types)'"
Enter
Sleep 4s

Type "rg 'return_type.*Utf8' crates/prism-query/src/infusion_udf.rs && echo VIOLATION || echo 'PASS: 0 matches'"
Enter
Sleep 2s
