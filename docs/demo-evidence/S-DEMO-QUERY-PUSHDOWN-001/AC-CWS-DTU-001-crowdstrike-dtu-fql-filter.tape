Output docs/demo-evidence/S-DEMO-QUERY-PUSHDOWN-001/AC-CWS-DTU-001-crowdstrike-dtu-fql-filter.gif
Output docs/demo-evidence/S-DEMO-QUERY-PUSHDOWN-001/AC-CWS-DTU-001-crowdstrike-dtu-fql-filter.webm

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

Type "# AC-CWS-DTU-001: CrowdStrike DTU honors FQL filter — filtered_count < unfiltered_count (LOAD-BEARING)"
Enter
Sleep 500ms

Type "cargo nextest run -p prism-dtu-crowdstrike --no-fail-fast"
Enter
Wait+Screen@120s /Summary/

Sleep 2s
