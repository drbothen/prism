# AC-006 / AC-007 — 4xx surfaces E-SENSOR-001; end-only filter is single less_or_equal
#
# AC-006: xDome returns 400 → adapter yields SensorError::HttpError{status:400};
#         NOT panic, NOT silent Vec::new(). Filter injection must be present first
#         (primary gate: body contains filter_by; secondary gate: error type is HttpError).
# AC-007: end_time=Some, start_time=None → SINGLE less_or_equal filter at end_time.
#         No compound AND, no synthetic greater_or_equal 7-day floor.
#         Tested with end_time="2026-01-01T00:00:00Z" (>7 months ago) to prove no floor injected.
#
# Tests run: RG-005 (error path), RG-006 (end-only)
# Wire-level assertions on outbound POST body + error variant.
# No external DTU required — wiremock mock server.
#
# Traces to: BC-2.01.013 §Postconditions Claroty audit_logs row (EC-01-032 end-only, error surface)
# Story: S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-006 / AC-007

Output "docs/demo-evidence/S-CLAROTY-AUDITLOG-TIMEBOX-001/AC-006-007-error-and-end-only-filter.gif"
Output "docs/demo-evidence/S-CLAROTY-AUDITLOG-TIMEBOX-001/AC-006-007-error-and-end-only-filter.webm"

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

Type "# AC-006/007: 4xx E-SENSOR-001 + end-only single less_or_equal (RG-005, RG-006)"
Enter
Sleep 400ms

Type "cargo nextest run -p prism-bin --test bc_2_01_013_claroty_audit_logs_layer2 -E 'test(filter_rejection_4xx) | test(end_only_single_less_or_equal)'"
Enter
Sleep 20s
Sleep 2s
