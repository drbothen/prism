# AC-006..009: Edge case unit tests (mock response payloads)
#
# AC-006: REQUIRED vulnerability_name absent → null row; no hard error; subsequent rows OK
# AC-007: Nullable count envelope (count=null) → empty-page halt; no null-deref
# AC-008: Nullable datetime fields (resolution_date=null, patch_install_date=null) → null passthrough
# AC-009: device_vulnerability_relevance_reasons=[] → stored as native JSON array []; NOT null
#
# Tests: RG-006 (test_BC_2_16_017_...required_vulnerability_name_absent_produces_null_row)
#        RG-007 (test_BC_2_16_017_...nullable_count_uses_empty_page_halt)
#        RG-008 (test_BC_2_16_017_...nullable_datetime_fields_pass_through)
#        RG-009 (test_BC_2_16_017_...empty_relevance_reasons_array_serialized)
# Traces to: BC-2.16.017 EC-016-017-001..004

Output "docs/demo-evidence/S-CLAROTY-DEVVULNREL-001/AC-006-009-edge-cases.gif"
Output "docs/demo-evidence/S-CLAROTY-DEVVULNREL-001/AC-006-009-edge-cases.webm"

Require cargo

Set Shell "zsh"
Set WaitTimeout 180s
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 16
Set Width 1200
Set Height 500
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 40ms

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-CLAROTY-DEVVULNREL-001"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 0.5s
Show

Type "# AC-006..009: edge case unit tests (REQUIRED absent, nullable count/datetime, JSON array) (RG-006..009)"
Enter
Sleep 400ms

Type `cargo nextest run -p prism-sensors -E 'test(required_vulnerability_name) + test(nullable_count) + test(nullable_datetime) + test(empty_relevance)' --no-fail-fast 2>&1 | tail -15`
Enter
Sleep 120s
Sleep 2s
