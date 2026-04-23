Output docs/demo-evidence/S-1.05/AC-009-normalization-error-context.gif
Output docs/demo-evidence/S-1.05/AC-009-normalization-error-context.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Catppuccin Mocha"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 24

# AC-9 (BC-2.02.011): Normalization error contains source record ID + specific field name.
# Error path: CrowdStrike detection missing detection_id → OcsfNormalizationFailed.
# Error path: Cyberint bad timestamp → OcsfTimestampParseError with field + raw value.

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-1.05-ocsf-field-mapping"
Enter
Sleep 500ms
Type "clear"
Enter
Sleep 300ms
Show

Type "# AC-9: Normalization errors include source record context and specific field name"
Enter
Sleep 800ms
Type "cargo test -p prism-ocsf test_BC_2_02_011_missing_detection_id_returns_normalization_error test_BC_2_02_011_error_contains_source_record_context_and_field_name 2>&1 | grep -E 'test.*ok|test.*FAILED|test result'"
Enter
Sleep 15s
