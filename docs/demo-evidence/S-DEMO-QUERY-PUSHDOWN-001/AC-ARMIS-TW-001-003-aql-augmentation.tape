Output docs/demo-evidence/S-DEMO-QUERY-PUSHDOWN-001/AC-ARMIS-TW-001-003-aql-augmentation.gif
Output docs/demo-evidence/S-DEMO-QUERY-PUSHDOWN-001/AC-ARMIS-TW-001-003-aql-augmentation.webm

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

Type "# AC-ARMIS-TW-001: AQL augmentation — after:/before: clauses appended"
Enter
Type "# AC-ARMIS-TW-003: Anti-double-filter guard"
Enter
Sleep 500ms

Type "cargo nextest run -p prism-query -E 'test(ac_armis_tw_001) or test(ac_armis_tw_003)' --no-fail-fast"
Enter
Wait+Screen@120s /Summary/

Sleep 2s
