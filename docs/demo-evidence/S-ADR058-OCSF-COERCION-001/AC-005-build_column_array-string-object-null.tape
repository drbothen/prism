# AC-005: build_column_array ColumnType::String arm returns null cell for Object input
# RG-006: test_build_column_array_string_type_object_input_returns_null_and_emits_warning
# Coverage: Path A (LIVE) — build_column_array in prism-bin::spec_driven_adapter
# Wire evidence: see AC-005-live-mcp-transcript.json
#   "description": null — key present, JSON null (BC-2.11.001 EC-11-079 explicit_nulls=true)
#   WARN column_coercion_failure column=description column_type=string actual_json_kind=object

Output docs/demo-evidence/S-ADR058-OCSF-COERCION-001/AC-005-build_column_array-string-object-null.gif
Output docs/demo-evidence/S-ADR058-OCSF-COERCION-001/AC-005-build_column_array-string-object-null.webm

Require cargo

Set Shell "zsh"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 18
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 50ms

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-ADR058-OCSF-COERCION-001"
Enter
Wait+Line /\$/
Type "clear"
Enter
Wait+Line /\$/
Show

# AC-005: String column + Object input -> null cell + column_coercion_failure warn (Path A)
# Before fix: Object fell through other=>Some(other.to_string()), serializing "{...}" as string
# After fix: explicit Value::Object(_) => None arm added BEFORE wildcard; warn emitted
Type "cargo nextest run -p prism-bin -E 'test(test_build_column_array_string_type_object_input_returns_null_and_emits_warning)' 2>&1 | tail -8"
Enter
Wait+Line /PASS|FAIL|Summary/
Sleep 2s
