# AC-003 — Claroty harness clone POST /api/v1/audit_log/get in both routers
# BC-2.16.013 v1.25 INV-HARNESS-ROUTE-PARITY
# Covers: logical router() AND network_router() (C-4)
# Auth model: 200 with any non-empty Bearer, 401 without (NOT 403 -- Armis only)
# Red Gate: POST /api/v1/audit_log/get -> 404 in either router
#
# DRIFT-SEC-TAPE-PATH-001 / DRIFT-D904-002: output paths relative to worktree root.

Output docs/demo-evidence/S-DEMO-HARNESS-CLONE-PARITY-001/AC-003-claroty-audit-log-auth.gif
Output docs/demo-evidence/S-DEMO-HARNESS-CLONE-PARITY-001/AC-003-claroty-audit-log-auth.webm

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

Type "# AC-003: Claroty POST /api/v1/audit_log/get in router() and network_router()"
Enter
Sleep 500ms
Type "cargo nextest run -p prism-dtu-harness --features dtu -E 'test(claroty_harness_audit_log_returns_200_with_bearer_401_without)' 2>&1 | tail -5"
Enter
Sleep 45s
