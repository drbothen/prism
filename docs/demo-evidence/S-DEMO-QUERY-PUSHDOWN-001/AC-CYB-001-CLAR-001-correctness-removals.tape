Output docs/demo-evidence/S-DEMO-QUERY-PUSHDOWN-001/AC-CYB-001-CLAR-001-correctness-removals.gif
Output docs/demo-evidence/S-DEMO-QUERY-PUSHDOWN-001/AC-CYB-001-CLAR-001-correctness-removals.webm

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

Type "# AC-CYB-001: Cyberint — no from_date/to_date/page_size (GET cursor-only)"
Enter
Type "# AC-CLAR-001: Claroty — body_template stays empty {}"
Enter
Sleep 500ms

Type "cargo nextest run -p prism-spec-engine -E 'test(ac_cyb_001) or test(ac_clar_001)' --no-fail-fast"
Enter
Wait+Screen@120s /Summary/

Sleep 2s
