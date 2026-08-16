# ALL ACs — S-CLAROTY-AUDITLOG-TIMEBOX-001 full 7-test Red Gate suite
#
# Runs all 7 Red Gate tests (RG-001 through RG-007) in a single pass.
# Provides the authoritative PASS evidence for all acceptance criteria.
#
# Covered ACs:
#   AC-001 (RG-001): default no-filter → greater_or_equal 7-day window in POST body
#   AC-002 (RG-002): explicit start_time honored — NOT silently capped to 7-day default
#   AC-003 (RG-003): both bounds → compound AND filter with "operands" key (EC-01-033)
#   AC-004 (RG-004): pipeline.rs JSON auto-parse + FQL/AQL backward-compat (Value::Object / Value::String)
#   AC-006 (RG-005): xDome 4xx → E-SENSOR-001 SensorError::HttpError{status:400}
#   AC-007 (RG-006): end-only filter → single less_or_equal, NO synthetic 7-day floor (EC-01-032)
#   AC-INDEX-CLARO-001 (RG-007): parser-surface WHERE timestamp > explicit → explicit bound in POST body
#
# DTU note: these tests use wiremock — no external Claroty DTU required.
# Live row-reduction behavior was validated against the live xDome API separately.
#
# Traces to: BC-2.01.013 v1.22, BC-2.16.013 v1.41
# Story: S-CLAROTY-AUDITLOG-TIMEBOX-001

Output "docs/demo-evidence/S-CLAROTY-AUDITLOG-TIMEBOX-001/ALL-ACs-full-suite.gif"
Output "docs/demo-evidence/S-CLAROTY-AUDITLOG-TIMEBOX-001/ALL-ACs-full-suite.webm"

Require cargo

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 550
Set Theme "Catppuccin Mocha"
Set Padding 24
Set TypingSpeed 40ms

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-CLAROTY-AUDITLOG-TIMEBOX-001"
Enter
Sleep 1s
Show

Type "# S-CLAROTY-AUDITLOG-TIMEBOX-001 — all 7 Red Gate tests (AC-001..AC-007 + AC-INDEX)"
Enter
Sleep 400ms

Type "cargo nextest run -p prism-bin --test bc_2_01_013_claroty_audit_logs_layer2 -p prism-spec-engine -E 'test(bc_2_01_013_claroty_audit_logs_layer2) | test(test_BC_2_16_013_pipeline_json_filter_string_parsed_to_value_object_backward_compat)'"
Enter
Sleep 30s
Sleep 2s
