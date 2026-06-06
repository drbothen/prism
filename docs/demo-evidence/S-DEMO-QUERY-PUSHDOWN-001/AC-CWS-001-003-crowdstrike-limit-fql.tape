Output docs/demo-evidence/S-DEMO-QUERY-PUSHDOWN-001/AC-CWS-001-003-crowdstrike-limit-fql.gif
Output docs/demo-evidence/S-DEMO-QUERY-PUSHDOWN-001/AC-CWS-001-003-crowdstrike-limit-fql.webm

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

Type "# AC-CWS-001: limit reaches DetectionListParams  AC-CWS-002: FQL both bounds  AC-CWS-003: no filter on empty predicate"
Enter
Sleep 500ms

Type "cargo nextest run -p prism-spec-engine -E 'test(ac_cws_001) or test(ac_cws_002) or test(ac_cws_003)' --no-fail-fast"
Enter
Wait+Screen@120s /Summary/

Sleep 2s
