# AC-002 — Armis harness clone AQL routing structural parity
# BC-2.16.013 v1.25 INV-HARNESS-ROUTE-PARITY
# Success path: in:devices -> non-empty device results; in:alerts -> distinct alert results
# C-7: structural parity — distinct entity types, not byte-exact with standalone
# Red Gate: GET /api/v1/search?aql=in:devices -> 404 (route not registered)
#
# DRIFT-SEC-TAPE-PATH-001 / DRIFT-D904-002: output paths relative to worktree root.

Output docs/demo-evidence/S-DEMO-HARNESS-CLONE-PARITY-001/AC-002-armis-search-aql-routing.gif
Output docs/demo-evidence/S-DEMO-HARNESS-CLONE-PARITY-001/AC-002-armis-search-aql-routing.webm

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

Type "# AC-002: AQL routing -- in:devices vs in:alerts return distinct non-empty arrays"
Enter
Sleep 500ms
Type "cargo nextest run -p prism-dtu-harness --features dtu -E 'test(armis_harness_search_aql_in_devices_returns_device_records)' 2>&1 | tail -5"
Enter
Sleep 45s
