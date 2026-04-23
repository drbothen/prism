Output docs/demo-evidence/S-1.14/AC-009-vp049-proptest.gif
Output docs/demo-evidence/S-1.14/AC-009-vp049-proptest.webm

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

# AC-10 / VP-049: proptest 1000 cases — enrich_single calls == unique value count
Type "# AC-10 (VP-049): proptest — source calls == unique values (1000 cases, N≤10K, K≤N)"
Enter
Sleep 500ms
Type "cargo test -q --package prism-spec-engine invariant_dedup 2>&1 | grep -E 'ok|FAILED|finished'"
Enter
Wait+Screen /finished/
Sleep 2s
