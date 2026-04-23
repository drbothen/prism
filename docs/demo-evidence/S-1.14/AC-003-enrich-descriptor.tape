Output docs/demo-evidence/S-1.14/AC-003-enrich-descriptor.gif
Output docs/demo-evidence/S-1.14/AC-003-enrich-descriptor.webm

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

# AC-3 / BC-2.19.001: enrich_descriptor for | enrich geoip ON device_ip → 4 output columns
Type "# AC-3 (BC-2.19.001): enrich_descriptor → 4 columns; E-INFUSE-001 for unknown name"
Enter
Sleep 500ms
Type "cargo test -q --package prism-spec-engine enrich_descriptor 2>&1 | grep -E 'ok|FAILED|finished'"
Enter
Wait+Screen /finished/
Sleep 2s
