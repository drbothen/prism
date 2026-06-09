# AC-001 — Armis harness clone GET /api/v1/search auth parity
# BC-2.16.013 v1.25 INV-HARNESS-ROUTE-PARITY
# Success path: 200 with real admin token, 403 without Bearer, 401 on wrong token
# Red Gate: GET /api/v1/search -> 404 (route not registered)
#
# DRIFT-SEC-TAPE-PATH-001 / DRIFT-D904-002: absolute path required for cd in Hide block
# (VHS has no cwd concept; path must be absolute. No /Users/<literal-name>/ hardcoded
# in any output path — output paths are relative to the worktree root.)

Output docs/demo-evidence/S-DEMO-HARNESS-CLONE-PARITY-001/AC-001-armis-search-auth.gif
Output docs/demo-evidence/S-DEMO-HARNESS-CLONE-PARITY-001/AC-001-armis-search-auth.webm

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

Type "# AC-001: Armis GET /api/v1/search -- 200 with real token, 403 without, 401 on wrong token"
Enter
Sleep 500ms
Type "cargo nextest run -p prism-dtu-harness --features dtu -E 'test(armis_harness_search_returns_200_with_bearer_403_without)' 2>&1 | tail -5"
Enter
Sleep 45s

Type "cargo nextest run -p prism-dtu-harness --features dtu -E 'test(armis_harness_search_401_on_wrong_token)' 2>&1 | tail -5"
Enter
Sleep 30s
