# AC-004 — Claroty harness audit_log response envelope matches standalone
# BC-2.16.013 v1.25 INV-HARNESS-ROUTE-PARITY
# Envelope: {"audit_log": [...], "total": N}
# 5-column check: id, action, actor, timestamp, resource
# Fixture embedded via include_str! of prism-dtu-claroty/fixtures/audit-log.json (C-1/C-8)
# Red Gate: POST /api/v1/audit_log/get -> 404 (route not registered)
#
# DRIFT-SEC-TAPE-PATH-001 / DRIFT-D904-002: output paths relative to worktree root.

Output docs/demo-evidence/S-DEMO-HARNESS-CLONE-PARITY-001/AC-004-claroty-audit-log-envelope.gif
Output docs/demo-evidence/S-DEMO-HARNESS-CLONE-PARITY-001/AC-004-claroty-audit-log-envelope.webm

Require cargo

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set PlaybackSpeed 1.0
Set WaitTimeout 120s

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-HARNESS-CLONE-PARITY-001"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-004: Claroty envelope {audit_log:[...], total:N} + 5-col check (id/action/actor/timestamp/resource)"
Enter
Sleep 500ms
Type "cargo nextest run -p prism-dtu-harness --features dtu -E 'test(claroty_harness_audit_log_response_envelope_matches_standalone)' 2>&1 | tail -5"
Enter
Sleep 45s
