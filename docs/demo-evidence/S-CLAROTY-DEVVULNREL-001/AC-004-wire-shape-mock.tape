# AC-004 (wire shape mock companion) + AC-006 (null-not-absent production path) + AC-009 (native JSON array passthrough)
#
# RG-011: test_BC_2_16_017_..._wire_shape_class_uid_2002_mock
#   Arrow RecordBatch-level assertions (NOT serialized JSON):
#   class_uid==2002 (Int32Array), finding_info_title column present by name,
#   raw_extensions StringArray with device_vulnerability_relevance_reasons as native
#   JSON array ["Software","Configuration"] (NOT stringified). No Tier-2 at top level.
#
# RG-010: test_BC_2_16_017_..._wire_shape_native_json_array_and_null_passthrough
#   Serialized-JSON assertions (arrow_json with explicit_nulls=true), 2-record mock:
#   Row0: class_uid=2002, finding_info_title="CVE-2024-EMPTY-REASONS",
#         time present+non-null+starts_with "2024-04-01T08:00:00" (MED-1 LOAD-BEARING),
#         device_vulnerability_relevance_reasons=[] native empty array NOT null NOT "[]".
#   Row1: finding_info_title=null (null-not-absent, BC-2.11.001 EC-11-079),
#         time=null (null-not-absent, MED-1 LOAD-BEARING, ADR-028 §D8-B),
#         device_vulnerability_relevance_reasons=["Network"] native array,
#         device_uid in raw_extensions.
#
# Tests: RG-011 (test_BC_2_16_017_...wire_shape_class_uid_2002_mock)
#        RG-010 (test_BC_2_16_017_...wire_shape_native_json_array_and_null_passthrough)
# Traces to: BC-2.16.017 §PC1 (class_uid=2002), §PC2 (Tier-1/Tier-2 wire shape),
#            §PC3 (composite PK in raw_extensions), AC-004, AC-006,
#            BC-2.11.001 EC-11-079 (null-not-absent), EC-016-017-004 (JSON array passthrough)

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
