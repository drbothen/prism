Output docs/demo-evidence/S-1.14/AC-010-error-path-duplicate-udf.gif
Output docs/demo-evidence/S-1.14/AC-010-error-path-duplicate-udf.webm

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

# Error paths: E-INFUSE-002 (duplicate UDF), E-INFUSE-003 (zero fields), E-RULE-012 format
Type "# Error paths: E-INFUSE-002 duplicate UDF, E-INFUSE-003 zero fields, E-RULE-012 format"
Enter
Sleep 500ms
Type "cargo test -q --package prism-spec-engine rejects 2>&1 | grep -E 'ok|FAILED|finished'"
Enter
Wait+Screen /finished/
Sleep 2s
