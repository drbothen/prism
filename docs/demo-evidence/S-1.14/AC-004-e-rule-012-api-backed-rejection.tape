Output docs/demo-evidence/S-1.14/AC-004-e-rule-012-api-backed-rejection.gif
Output docs/demo-evidence/S-1.14/AC-004-e-rule-012-api-backed-rejection.webm

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

# AC-4 / BC-2.19.003: is_api_backed() identifies plugin infusion UDFs for E-RULE-012
Type "# AC-4 (BC-2.19.003): is_api_backed() + E-RULE-012 detection rule rejection"
Enter
Sleep 500ms
Type "cargo test -q --package prism-spec-engine BC_2_19_003 2>&1 | grep -E 'ok|FAILED|finished'"
Enter
Wait+Screen /finished/
Sleep 2s
