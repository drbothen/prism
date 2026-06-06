Output docs/demo-evidence/S-DEMO-QUERY-PUSHDOWN-001/AC-EQUIV-001-EC-009-e2e-result-equivalence.gif
Output docs/demo-evidence/S-DEMO-QUERY-PUSHDOWN-001/AC-EQUIV-001-EC-009-e2e-result-equivalence.webm

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

Type "# AC-EQUIV-001: Result-equivalence via real run_materialization_pipeline (BC-2.11.007)"
Enter
Type "# EC-009: Inclusive boundary (>= / <=) — boundary record present in push-down result"
Enter
Sleep 500ms

Type "cargo nextest run -p prism-bin -E 'test(ac_equiv_001) or test(adv_p08)' --no-fail-fast"
Enter
Wait+Screen@180s /Summary/

Sleep 2s
