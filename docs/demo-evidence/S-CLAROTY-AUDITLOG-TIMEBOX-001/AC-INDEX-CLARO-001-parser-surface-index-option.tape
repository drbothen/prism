# AC-INDEX-CLARO-001 — audit_logs.timestamp OPTIONS=INDEX enables parser-surface push-down
#
# Proves that a PrismQL WHERE timestamp > '2025-01-01T00:00:00Z' predicate
# (parser-surface, SAP-3 reachability) extracts the explicit bound from the AST
# and sends it as the filter_by value — NOT the 7-day default.
#
# Without options=["INDEX"] on audit_logs.timestamp, extract_time_window_from_ast
# returns (None, None); build_claroty_audit_filter_by(None, None) injects the 7-day
# default, silently discarding the user's WHERE predicate (SOUL.md §4 silent-wrong-result).
#
# Test run: RG-007 (end-to-end parser-surface test; SAP-3 compliant)
# Call graph: PrismQL string → PrismQlParser::parse → extract_time_window_from_ast
#             → QueryParams.start_time → build_claroty_audit_filter_by → wiremock captures POST body
#             → assert filter_by.value equals explicit bound, NOT now−604800s
#
# Traces to: BC-2.01.013 EC-01-034; BC-2.16.013 v1.41 §1 audit_logs.timestamp INDEX prerequisite
# Story: S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-INDEX-CLARO-001

Output "docs/demo-evidence/S-CLAROTY-AUDITLOG-TIMEBOX-001/AC-INDEX-CLARO-001-parser-surface-index-option.gif"
Output "docs/demo-evidence/S-CLAROTY-AUDITLOG-TIMEBOX-001/AC-INDEX-CLARO-001-parser-surface-index-option.webm"

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

Type "# AC-INDEX-CLARO-001: parser-surface WHERE timestamp > explicit bound honored (RG-007)"
Enter
Sleep 400ms

Type "cargo nextest run -p prism-bin --test bc_2_01_013_claroty_audit_logs_layer2 -E 'test(timestamp_index_option_required_for_pushdown_eligibility)'"
Enter
Sleep 20s
Sleep 2s
