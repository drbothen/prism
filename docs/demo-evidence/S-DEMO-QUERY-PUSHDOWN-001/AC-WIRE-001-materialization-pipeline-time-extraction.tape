Output docs/demo-evidence/S-DEMO-QUERY-PUSHDOWN-001/AC-WIRE-001-materialization-pipeline-time-extraction.gif
Output docs/demo-evidence/S-DEMO-QUERY-PUSHDOWN-001/AC-WIRE-001-materialization-pipeline-time-extraction.webm

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

Type "# AC-WIRE-001 + AC-WIRE-001b: run_materialization_pipeline extracts time-window from PrismQL AST (ADR-033 T1)"
Enter
Sleep 500ms

Type "cargo nextest run -p prism-query -E 'test(ac_wire_001)' --no-fail-fast"
Enter
Wait+Screen@120s /Summary/

Sleep 2s
