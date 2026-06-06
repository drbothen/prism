Output docs/demo-evidence/S-DEMO-QUERY-PUSHDOWN-001/AC-CWS-002-e2e-fql-via-materialization.gif
Output docs/demo-evidence/S-DEMO-QUERY-PUSHDOWN-001/AC-CWS-002-e2e-fql-via-materialization.webm

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

Type "# AC-CWS-002 (e2e): FQL both start+end via run_materialization_pipeline"
Enter
Type "# Wire-level: /dtu/filter-log receives combined created_timestamp:>'T'+created_timestamp:<'T'"
Enter
Sleep 500ms

Type "cargo nextest run -p prism-bin -E 'test(ac_cws_002) or test(adv_p02_e2e_crowdstrike)' --no-fail-fast"
Enter
Wait+Screen@180s /Summary/

Sleep 2s
