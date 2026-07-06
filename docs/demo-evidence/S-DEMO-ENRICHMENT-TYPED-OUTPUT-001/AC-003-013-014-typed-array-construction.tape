# AC-003 — invoke_async_with_args() builds correctly typed Arrow arrays per output_arrow_type()
# AC-013 — NVD cvss_base_score UDF returns Float64Array; numeric comparison correct
# AC-014 — datetime-output UDF returns Timestamp(µs,UTC) via parse_datetime_to_micros
#
# Demonstrates:
#   RGT-003: test_invoke_async_with_args_returns_int64_array_for_integer_output_type PASS
#   RGT-004: test_invoke_async_with_args_returns_float64_array_for_float_output_type PASS
#   RGT-005: test_invoke_async_with_args_returns_boolean_array_for_boolean_output_type PASS
#   RGT-006: test_invoke_async_with_args_returns_timestamp_microsecond_array_for_datetime_output_type PASS
#
# BC: BC-2.19.001 v2.2 INV-ENRICH-TYPED-001 clause 2; TV-19-001-typed-*; ADR-051 D1/D5
# Crate: prism-query
# Branch: feature/S-DEMO-ENRICHMENT-TYPED-OUTPUT-001 HEAD 622bd2fa

Output docs/demo-evidence/S-DEMO-ENRICHMENT-TYPED-OUTPUT-001/AC-003-013-014-typed-array-construction.gif
Output docs/demo-evidence/S-DEMO-ENRICHMENT-TYPED-OUTPUT-001/AC-003-013-014-typed-array-construction.webm

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
Type "cargo nextest run -p prism-query -E 'test(invoke_async_with_args_returns)' 2>/dev/null"
Enter
Sleep 10s
Show

Type "# AC-003/013/014: Int64, Float64, Bool, Timestamp(µs,UTC) typed Arrow arrays"
Enter
Sleep 300ms
Type "cargo nextest run -p prism-query -E 'test(invoke_async_with_args_returns)'"
Enter
Sleep 4s
