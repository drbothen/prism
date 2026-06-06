Output docs/demo-evidence/S-DEMO-QUERY-PUSHDOWN-001/AC-ARMIS-TW-002-004-dtu-filter-equivalence.gif
Output docs/demo-evidence/S-DEMO-QUERY-PUSHDOWN-001/AC-ARMIS-TW-002-004-dtu-filter-equivalence.webm

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

Type "# AC-ARMIS-TW-002: Armis DTU filters fixture by time window (LOAD-BEARING: filtered < unfiltered)"
Enter
Type "# AC-ARMIS-TW-004: Result-equivalence — push-down vs DataFusion post-filter identical"
Enter
Sleep 500ms

Type "cargo nextest run -p prism-spec-engine -E 'test(armis_tw_002) or test(armis_tw_004)' --no-fail-fast"
Enter
Wait+Screen@120s /Summary/

Sleep 2s
