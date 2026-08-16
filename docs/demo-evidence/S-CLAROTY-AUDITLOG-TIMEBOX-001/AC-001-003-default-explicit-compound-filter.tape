# AC-001 / AC-002 / AC-003 — Claroty audit_logs filter_by injection
#
# AC-001: no time filter → POST body contains filter_by.operation="greater_or_equal"
#         with value ≈ now−7d (bounded default; never unbounded)
# AC-002: explicit start_time → filter_by.value equals the user-supplied bound, NOT 7-day cap
# AC-003: both start_time + end_time → compound AND filter with "operands" key (NOT "conditions")
#
# Tests run: RG-001, RG-002, RG-003
# Wire-level assertions: parse outbound POST body JSON; assert field/operation/value shape.
# No external DTU required — wiremock mock server captures outbound HTTP.
#
# Traces to: BC-2.01.013 §Postconditions Claroty audit_logs row (EC-01-030, EC-01-031, EC-01-033)
# Story: S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-001 / AC-002 / AC-003

Output "docs/demo-evidence/S-CLAROTY-AUDITLOG-TIMEBOX-001/AC-001-003-default-explicit-compound-filter.gif"
Output "docs/demo-evidence/S-CLAROTY-AUDITLOG-TIMEBOX-001/AC-001-003-default-explicit-compound-filter.webm"

Require cargo

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 500
Set Theme "Catppuccin Mocha"
Set Padding 24
Set TypingSpeed 40ms

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-CLAROTY-AUDITLOG-TIMEBOX-001"
Enter
Sleep 1s
Show

Type "# AC-001/002/003: Claroty audit_logs filter_by injection (RG-001, RG-002, RG-003)"
Enter
Sleep 400ms

Type "cargo nextest run -p prism-bin --test bc_2_01_013_claroty_audit_logs_layer2 -E 'test(no_filter_injects_default) | test(explicit_start_time_honored) | test(both_bounds_compound_and)'"
Enter
Sleep 30s
Sleep 2s
