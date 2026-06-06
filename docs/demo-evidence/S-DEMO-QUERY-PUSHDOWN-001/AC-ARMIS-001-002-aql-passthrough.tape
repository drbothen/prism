Output docs/demo-evidence/S-DEMO-QUERY-PUSHDOWN-001/AC-ARMIS-001-002-aql-passthrough.gif
Output docs/demo-evidence/S-DEMO-QUERY-PUSHDOWN-001/AC-ARMIS-001-002-aql-passthrough.webm

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

Type "# AC-ARMIS-001: AQL passthrough — no maxResults/timeFrame injected"
Enter
Type "# AC-ARMIS-002: Only aql/offset/limit params"
Enter
Sleep 500ms

Type "cargo nextest run -p prism-spec-engine -E 'test(ac_armis_001) or test(ac_armis_002)' --no-fail-fast"
Enter
Wait+Screen@120s /Summary/

Sleep 2s
