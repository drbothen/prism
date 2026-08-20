# AC-004: map_record emits column_coercion_failure warn at demotion
# RG-005: test_map_record_string_object_input_demotes_to_raw_extensions_and_emits_warning
# Coverage: Path B — ColumnMapper::map_record in prism-spec-engine (in-crate unit test)
# Note: tracing-test subscriber captures prism_spec_engine=trace events; in-crate placement required
# Path-A live MCP evidence: see AC-004-live-mcp-transcript.json

Output docs/demo-evidence/S-ADR058-OCSF-COERCION-001/AC-004-map_record-coercion-failure-warn.gif
Output docs/demo-evidence/S-ADR058-OCSF-COERCION-001/AC-004-map_record-coercion-failure-warn.webm

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
Sleep 2s
Type "clear"
Enter
Sleep 2s
Show

# AC-004: map_record places Object-valued String-column field in raw_extensions
# AND emits tracing::warn!(event_type = "column_coercion_failure")
# Before fix: no warn emitted — silent demotion violated BC-2.02.011
# After fix: warn includes column/column_type/actual_json_kind fields
Type "cargo nextest run -p prism-spec-engine -E 'test(test_map_record_string_object_input_demotes_to_raw_extensions_and_emits_warning)' 2>&1 | tail -8"
Enter
Sleep 15s
Sleep 2s
