Output docs/demo-evidence/S-DEMO-QUERY-PUSHDOWN-001/AC-ARMIS-TW-005-e2e-aql-log-ignored.gif
Output docs/demo-evidence/S-DEMO-QUERY-PUSHDOWN-001/AC-ARMIS-TW-005-e2e-aql-log-ignored.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set Padding 24
Set Shell "bash"

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-QUERY-PUSHDOWN-001"
Enter
Sleep 500ms
Show

Type "# AC-ARMIS-TW-005: E2E aql-log contains augmented AQL"
Enter
Type "# Note: runs via --run-ignored ignored-only (requires DTU + prism binary)"
Enter
Sleep 500ms

Type "cargo nextest run -p prism-spec-engine -E 'test(armis_tw_005)' --run-ignored ignored-only"
Enter
Wait+Screen@120s /Summary/

Sleep 2s
