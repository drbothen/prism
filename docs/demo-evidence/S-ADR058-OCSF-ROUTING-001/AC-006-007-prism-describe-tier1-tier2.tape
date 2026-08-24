# VHS Demo — AC-006 + AC-007 + AC-008 + AC-015
# AC-006: prism_describe returns flattened OCSF names in ColumnDescriptors
# AC-007: prism_describe emits raw_extensions descriptor; no individual Tier-2 ColumnDescriptors
# AC-008: prism_describe emits synthesized class_uid and _sensor ColumnDescriptors last
# AC-015: prism_describe names agree with ocsf_projected_column_names helper
# Tests: RG-007, RG-025, RG-026, RG-027, RG-028

Output docs/demo-evidence/S-ADR058-OCSF-ROUTING-001/AC-006-007-prism-describe-tier1-tier2.gif
Output docs/demo-evidence/S-ADR058-OCSF-ROUTING-001/AC-006-007-prism-describe-tier1-tier2.webm

Require cargo

Set Shell "zsh"
Set WaitTimeout 60s
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 17
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 40ms

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-ADR058-OCSF-ROUTING-001"
Enter
Sleep 1.5s
Type "clear"
Enter
Sleep 0.5s
Show

# All four prism_describe OCSF routing tests (Tier-1 flattened name, raw_extensions, synthesized descriptors, projection agreement)
Type "cargo nextest run -p prism-mcp -E 'test(prism_describe_ocsf_column_naming) | test(RG_Q_015_prism_describe)' 2>&1 | tail -10"
Enter
Sleep 20s
Sleep 2s
