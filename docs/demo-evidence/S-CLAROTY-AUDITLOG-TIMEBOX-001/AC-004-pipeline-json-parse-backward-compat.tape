# AC-004 — pipeline.rs JSON-object filter strings auto-parsed; plain FQL/AQL strings unchanged
#
# Proves:
#   - JSON-object string starting with '{' → serde_json::Value::Object in step_vars (parsed path)
#   - FQL string 'created_timestamp:>2026-01-01' → Value::String (else-branch backward-compat gate)
#   - The else-branch assertion is a positive assert_eq!(result, Value::String(...)),
#     NOT merely absence of panic (AC-004 §Test requirement).
#
# Test run: RG-004 (pipeline.rs in-module unit test)
# No HTTP required — pure function under test.
#
# Traces to: BC-2.16.013 §Postconditions §1 Layer-2 block — pipeline.rs auto-parse clause
#            BC-2.01.013 postcondition — backward-compat invariant (EC-003, EC-004)
# Story: S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-004

Output "docs/demo-evidence/S-CLAROTY-AUDITLOG-TIMEBOX-001/AC-004-pipeline-json-parse-backward-compat.gif"
Output "docs/demo-evidence/S-CLAROTY-AUDITLOG-TIMEBOX-001/AC-004-pipeline-json-parse-backward-compat.webm"

Require cargo

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 480
Set Theme "Catppuccin Mocha"
Set Padding 24
Set TypingSpeed 40ms

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-CLAROTY-AUDITLOG-TIMEBOX-001"
Enter
Sleep 1s
Show

Type "# AC-004: pipeline.rs JSON-filter auto-parse + FQL/AQL backward-compat (RG-004)"
Enter
Sleep 400ms

Type "cargo nextest run -p prism-spec-engine -E 'test(test_BC_2_16_013_pipeline_json_filter_string_parsed_to_value_object_backward_compat)'"
Enter
Sleep 20s
Sleep 2s
