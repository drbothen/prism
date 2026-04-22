Output docs/demo-evidence/S-1.14/AC-007-csv-source-backend.gif
Output docs/demo-evidence/S-1.14/AC-007-csv-source-backend.webm

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

# AC-7: CSV source backend — asset_inventory spec loads with asset_owner + asset_department UDFs
Type "# AC-7: CSV source backend — asset_inventory → asset_owner + asset_department UDFs"
Enter
Sleep 500ms
Type "cargo test -q --package prism-spec-engine ac_7 2>&1 | grep -E 'ok|FAILED|finished'"
Enter
Wait+Screen /finished/
Sleep 2s
