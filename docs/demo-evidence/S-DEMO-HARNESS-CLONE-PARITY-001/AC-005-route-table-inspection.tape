# AC-005 — Module-doc route tables: armis.rs and claroty.rs register the parity routes
# BC-2.16.013 v1.25 INV-HARNESS-ROUTE-PARITY
# Verified by source inspection: grep confirms route registration in router() / network_router()
# Success path: GET /api/v1/search in armis.rs router(), POST /api/v1/audit_log/get in both claroty routers
#
# DRIFT-SEC-TAPE-PATH-001 / DRIFT-D904-002: output paths relative to worktree root.

Output docs/demo-evidence/S-DEMO-HARNESS-CLONE-PARITY-001/AC-005-route-table-inspection.gif
Output docs/demo-evidence/S-DEMO-HARNESS-CLONE-PARITY-001/AC-005-route-table-inspection.webm

Require grep

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set PlaybackSpeed 1.0
Set WaitTimeout 30s

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-HARNESS-CLONE-PARITY-001"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-005: Confirm /api/v1/search registered in armis::router()"
Enter
Sleep 300ms
Type "grep -n 'api/v1/search' crates/prism-dtu-harness/src/clones/armis.rs"
Enter
Sleep 2s

Type "# AC-005: Confirm /api/v1/audit_log/get in claroty::router() and network_router()"
Enter
Sleep 300ms
Type "grep -n 'audit_log/get' crates/prism-dtu-harness/src/clones/claroty.rs"
Enter
Sleep 2s
