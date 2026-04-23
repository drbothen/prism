Output docs/demo-evidence/S-1.14/AC-001-infusion-registry-udf-export.gif
Output docs/demo-evidence/S-1.14/AC-001-infusion-registry-udf-export.webm

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

# AC-1 / BC-2.19.001: InfusionRegistry loads geoip spec → exactly 4 UDF descriptors exported
Type "# AC-1 (BC-2.19.001): geoip spec → 4 InfusionUdfDescriptors"
Enter
Sleep 500ms
Type "cargo test -q --package prism-spec-engine BC_2_19_001 2>&1 | grep -E 'ok|FAILED|finished'"
Enter
Wait+Screen /finished/
Sleep 2s
