# AC-004 (wire shape mock companion) + AC-009 (native JSON array passthrough at wire level)
#
# AC-004 mock companion: test_BC_2_16_017_..._wire_shape_class_uid_2002_mock
#   Mock response with seeded device vulnerability row. Serialized JSON wire output contains:
#   class_uid=2002, finding_info_title present, time present, raw_extensions present as
#   JSON object with device_uid + vulnerability_id keys. No Tier-2 keys at row root.
#
# AC-009 EC-004-WIRE: test_BC_2_16_017_..._wire_shape_native_json_array_and_null_passthrough
#   device_vulnerability_relevance_reasons=[] serialized as native JSON array [] (NOT string).
#   null passthrough at wire level for nullable Tier-2 fields.
#
# Tests: RG-011 (test_BC_2_16_017_...wire_shape_class_uid_2002_mock)
#        RG-010 (test_BC_2_16_017_...wire_shape_native_json_array_and_null_passthrough)
# Traces to: BC-2.16.017 §PC1 (class_uid=2002), §PC2 (Tier-1/Tier-2 wire shape),
#            §PC3 (composite PK in raw_extensions), EC-016-017-004 (JSON array passthrough)

Output "docs/demo-evidence/S-CLAROTY-DEVVULNREL-001/AC-004-wire-shape-mock.gif"
Output "docs/demo-evidence/S-CLAROTY-DEVVULNREL-001/AC-004-wire-shape-mock.webm"

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

Type "# AC-004/009: wire-shape mock (class_uid=2002, Tier-1, composite PK, JSON array) (RG-010, RG-011)"
Enter
Sleep 400ms

Type `cargo nextest run -p prism-bin -E 'test(wire_shape)' --no-fail-fast 2>&1 | tail -15`
Enter
Sleep 120s
Sleep 2s
