Output docs/demo-evidence/S-1.14/AC-002-query-scoped-dedup-cache.gif
Output docs/demo-evidence/S-1.14/AC-002-query-scoped-dedup-cache.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set Shell "zsh"
Set WaitTimeout 120s

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-1.14-infusion-specs"
Enter
Type "clear"
Enter
Show

# AC-2 / BC-2.19.002: QueryScopedInfusionCache — 10K events, 200 unique IPs → 200 source calls
Type "# AC-2 (BC-2.19.002): 10K events, 200 unique IPs → 200 enrich_single calls"
Enter
Sleep 500ms
Type "cargo test -q --package prism-spec-engine BC_2_19_002 2>&1 | grep -E 'ok|FAILED|finished'"
Enter
Wait+Screen /finished/
Sleep 2s
